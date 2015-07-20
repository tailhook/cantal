use std::mem::replace;
use std::net::{Ipv4Addr, SocketAddr};
use std::ops::DerefMut;
use std::sync::{Arc, RwLock};
use std::collections::HashMap;

use mio::{Token, Timeout, EventSet};
use mio::tcp::TcpStream;
use mio::util::Slab;
use time::{SteadyTime};
use rand::thread_rng;
use rand::distributions::{IndependentSample, Range};

use super::server::Context;
use super::scan::time_ms;
use super::websock::Beacon;
use super::websock::InputMessage as OutputMessage;
use super::websock::OutputMessage as InputMessage;
use super::deps::{Dependencies, LockedDeps};
use super::server::Timer::{ReconnectPeer, ResetPeer};
use super::p2p::GossipStats;
use self::owebsock::WebSocket;

mod owebsock;


const SLAB_START: usize = 1000000000;
const MAX_OUTPUT_CONNECTIONS: usize = 4096;
const HANDSHAKE_TIMEOUT: u64 = 30000;
const MESSAGE_TIMEOUT: u64 = 15000;


pub type PeerHolder = Arc<RwLock<Peers>>;


pub struct Peers {
    start_time: SteadyTime,
    pub connected: usize,
    pub addresses: HashMap<SocketAddr, Token>,
    pub peers: Slab<Peer>,
}

enum Connection {
    WebSock(WebSocket),
    Sleep,
}

pub struct Peer {
    pub addr: SocketAddr,
    connection: Connection,
    timeout: Timeout,
    pub last_beacon: Option<(u64, Beacon)>,
}

impl Peer {
    pub fn connected(&self) -> bool {
        match self.connection {
            Connection::WebSock(ref w) if !w.handshake => true,
            _ => false,
        }
    }
}

pub fn start(ctx: &mut Context) {
    if ctx.deps.get::<PeerHolder>().is_some() {
        return; // already started
    }
    let range = Range::new(5, 150);
    let mut rng = thread_rng();
    let peers:Vec<_>;
    peers = ctx.deps.read::<GossipStats>().peers.keys().cloned().collect();
    let mut data = Peers {
        start_time: SteadyTime::now(),
        peers: Slab::new_starting_at(Token(SLAB_START),
                                     MAX_OUTPUT_CONNECTIONS),
        connected: 0,
        addresses: HashMap::new(),
    };
    for addr in peers {
        if let Some(tok) = data.peers.insert_with(|tok| Peer {
            addr: addr,
            last_beacon: None,
            connection: Connection::Sleep,
            timeout: ctx.eloop.timeout_ms(ReconnectPeer(tok),
                range.ind_sample(&mut rng)).unwrap(),
        }) {
            data.addresses.insert(addr, tok);
        } else {
            error!("Too many peers");
        }
    }
    ctx.deps.insert(Arc::new(RwLock::new(data)));
}

pub fn add_peer(addr: SocketAddr, ctx: &mut Context) {
    let range = Range::new(5, 150);
    let mut rng = thread_rng();
    if ctx.deps.get::<PeerHolder>().is_none() {
        // Remote handling is not enabled ATM
        return;
    }
    let mut data = ctx.deps.write::<Peers>();
    if data.addresses.contains_key(&addr) {
        return;
    }
    let ref mut eloop = ctx.eloop;
    if let Some(tok) = data.peers.insert_with(|tok| Peer {
        addr: addr,
        last_beacon: None,
        connection: Connection::Sleep,
        timeout: eloop.timeout_ms(ReconnectPeer(tok),
            range.ind_sample(&mut rng)).unwrap(),
    }) {
        data.addresses.insert(addr, tok);
    } else {
        error!("Too many peers");
    }
}

pub fn reconnect_peer(tok: Token, ctx: &mut Context) {
    let mut data = ctx.deps.write::<Peers>();
    if let Some(ref mut peer) = data.peers.get_mut(tok) {
        match peer.connection {
            Connection::WebSock(_) => unreachable!(),
            Connection::Sleep => {}
        }
        let range = Range::new(1000, 2000);
        let mut rng = thread_rng();
        if let Ok(conn) = WebSocket::connect(peer.addr) {
            match conn.register(tok, ctx.eloop) {
                Ok(_) => {
                    peer.connection = Connection::WebSock(conn);
                    peer.timeout = ctx.eloop.timeout_ms(ResetPeer(tok),
                        HANDSHAKE_TIMEOUT).unwrap();
                }
                _ => {
                    peer.connection = Connection::Sleep;
                    peer.timeout = ctx.eloop.timeout_ms(ReconnectPeer(tok),
                        range.ind_sample(&mut rng)).unwrap();
                }
            }
        } else {
            peer.connection = Connection::Sleep;
            peer.timeout = ctx.eloop.timeout_ms(ReconnectPeer(tok),
                range.ind_sample(&mut rng)).unwrap();
        }
    }
}

pub fn reset_peer(tok: Token, ctx: &mut Context) {
    let mut data = ctx.deps.write::<Peers>();
    if let Some(ref mut peer) = data.peers.get_mut(tok) {
        let sock = match replace(&mut peer.connection, Connection::Sleep) {
            Connection::WebSock(sock) => sock,
            Connection::Sleep => unreachable!(),
        };
        sock.deregister(ctx.eloop)
            .map_err(|e| error!("Error on deregister: {}", e))
            .ok();
        let range = Range::new(1000, 2000);
        let mut rng = thread_rng();
        peer.timeout = ctx.eloop.timeout_ms(ReconnectPeer(tok),
            range.ind_sample(&mut rng)).unwrap();
    }
}

pub fn try_io(tok: Token, ev: EventSet, ctx: &mut Context) -> bool
{
    let dataref = ctx.deps.get::<PeerHolder>().unwrap().clone();
    let mut dataguard = dataref.write().unwrap();
    let ref mut data = dataguard.deref_mut();
    if let Some(ref mut peer) = data.peers.get_mut(tok) {
        let to_close = {
            let ref mut sock = match peer.connection {
                Connection::WebSock(ref mut sock) => sock,
                Connection::Sleep => unreachable!(),
            };
            let old = sock.handshake;
            let mut to_close;
            if let Some(messages) = sock.events(ev, tok, ctx) {
                if messages.len() > 0 {
                    assert!(ctx.eloop.clear_timeout(peer.timeout));
                    peer.timeout = ctx.eloop.timeout_ms(ResetPeer(tok),
                        MESSAGE_TIMEOUT).unwrap();
                }
                for msg in messages {
                    match msg {
                        InputMessage::Beacon(b) => {
                            peer.last_beacon = Some((time_ms(), b));
                        }
                        InputMessage::NewPeer(p) => {
                            // TODO(tailhook) process it
                            debug!("New peer from websock {:?}", p);
                        }
                    }
                }
                to_close = false;
            } else {
                to_close = true;
            }
            if old &&  !to_close && !sock.handshake {
                data.connected += 1;
                assert!(ctx.eloop.clear_timeout(peer.timeout));
                peer.timeout = ctx.eloop.timeout_ms(ResetPeer(tok),
                    MESSAGE_TIMEOUT).unwrap();
            } else if !old && to_close {
                data.connected -= 1;
            }
            to_close
        };
        if to_close {
            let range = Range::new(5, 150);
            let mut rng = thread_rng();
            peer.connection = Connection::Sleep;
            assert!(ctx.eloop.clear_timeout(peer.timeout));
            peer.timeout = ctx.eloop.timeout_ms(ReconnectPeer(tok),
                    range.ind_sample(&mut rng)).unwrap();
        }
        return true;
    } else {
        return false;
    }
    //data.peers.remove(tok)
    //return true;
}
