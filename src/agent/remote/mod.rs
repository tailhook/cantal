use std::net::{Ipv4Addr, SocketAddr};
use std::ops::DerefMut;
use std::sync::{Arc, RwLock};
use std::collections::HashMap;

use mio::{Token, Timeout, EventSet};
use mio::tcp::TcpStream;
use mio::util::Slab;
use time::SteadyTime;
use rand::thread_rng;
use rand::distributions::{IndependentSample, Range};

use super::server::Context;
use super::deps::{Dependencies, LockedDeps};
use super::server::Timer::ReconnectPeer;
use super::p2p::GossipStats;
use self::owebsock::WebSocket;

mod owebsock;


const SLAB_START: usize = 1000000000;
const MAX_OUTPUT_CONNECTIONS: usize = 4096;


pub type PeerHolder = Arc<RwLock<Peers>>;


pub struct Peers {
    start_time: SteadyTime,
    pub connected: usize,
    pub addresses: HashMap<SocketAddr, Token>,
    peers: Slab<Peer>,
}

enum Connection {
    WebSock(WebSocket),
    Timeout(Timeout),
}

pub struct Peer {
    addr: SocketAddr,
    connection: Connection,
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
            connection: Connection::Timeout(
                ctx.eloop.timeout_ms(ReconnectPeer(tok),
                                     range.ind_sample(&mut rng)).unwrap()),
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
        connection: Connection::Timeout(
            eloop.timeout_ms(ReconnectPeer(tok),
                range.ind_sample(&mut rng)).unwrap()),
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
            Connection::Timeout(_) => {}
        }
        let range = Range::new(5, 150);
        let mut rng = thread_rng();
        if let Ok(conn) = WebSocket::connect(peer.addr) {
            match conn.register(tok, ctx.eloop) {
                Ok(_) => {
                    peer.connection = Connection::WebSock(conn);
                }
                _ => {
                    peer.connection = Connection::Timeout(
                        ctx.eloop.timeout_ms(ReconnectPeer(tok),
                            range.ind_sample(&mut rng)).unwrap());
                }
            }
        } else {
            peer.connection = Connection::Timeout(
                ctx.eloop.timeout_ms(ReconnectPeer(tok),
                    range.ind_sample(&mut rng)).unwrap());
        }
    }
}

pub fn try_io(tok: Token, ev: EventSet, ctx: &mut Context) -> bool
{
    let dataref = ctx.deps.get::<PeerHolder>().unwrap().clone();
    let mut dataguard = dataref.write().unwrap();
    let ref mut data = dataguard.deref_mut();
    if let Some(ref mut peer) = data.peers.get_mut(tok) {
        let result = {
            let ref mut sock = match peer.connection {
                Connection::WebSock(ref mut sock) => sock,
                Connection::Timeout(_) => unreachable!(),
            };
            let old = sock.handshake;
            let res = sock.events(ev, tok, ctx);
            if old && res && !sock.handshake {
                data.connected += 1;
            } else if !old && !res {
                data.connected -= 1;
            }
            res
        };
        if !result {
            let range = Range::new(5, 150);
            let mut rng = thread_rng();
            peer.connection = Connection::Timeout(
                ctx.eloop.timeout_ms(ReconnectPeer(tok),
                    range.ind_sample(&mut rng)).unwrap());
        }
        return true;
    } else {
        return false;
    }
    //data.peers.remove(tok)
    //return true;
}
