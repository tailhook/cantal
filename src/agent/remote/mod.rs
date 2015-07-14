use std::net::{Ipv4Addr, SocketAddr};
use std::sync::{Arc, RwLock};
use std::collections::HashMap;

use mio::Token;
use mio::Timeout;
use mio::tcp::TcpStream;
use mio::util::Slab;
use rand::thread_rng;
use rand::distributions::{IndependentSample, Range};

use super::server::Context;
use super::server::Timer::ReconnectPeer;
use self::owebsock::WebSocket;

mod owebsock;


const SLAB_START: usize = 1000000000;
const MAX_OUTPUT_CONNECTIONS: usize = 4096;


pub type Peers = Arc<RwLock<PeerData>>;


pub struct PeerData {
    addresses: HashMap<SocketAddr, Token>,
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
    let range = Range::new(5, 150);
    let mut rng = thread_rng();
    let peers: Vec<SocketAddr> = {
        let stats = ctx.stats.read().unwrap();
        if stats.peers.is_some() {
            return;
        }
        let gossip = stats.gossip.clone();
        let vec = gossip.read().unwrap().peers.keys().cloned().collect();
        vec
    };
    let mut data = PeerData {
        peers: Slab::new_starting_at(Token(SLAB_START),
                                     MAX_OUTPUT_CONNECTIONS),
        addresses: HashMap::new(),
    };
    for addr in peers {
        data.peers.insert_with(|tok| Peer {
            addr: addr,
            connection: Connection::Timeout(
                ctx.eloop.timeout_ms(ReconnectPeer(tok),
                                     range.ind_sample(&mut rng)).unwrap()),
        });
    }
    ctx.stats.write().unwrap().peers = Some(Arc::new(RwLock::new(data)));
}

pub fn add_peer(addr: SocketAddr, ctx: &mut Context) {
    let range = Range::new(5, 150);
    let mut rng = thread_rng();
    if let Some(dataref) = ctx.stats.read().unwrap().peers.clone() {
        let mut data = dataref.write().unwrap();
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
}

pub fn reconnect_peer(tok: Token, ctx: &mut Context) {
    if let Some(dataref) = ctx.stats.read().unwrap().peers.clone() {
        let mut data = dataref.write().unwrap();
        if let Some(ref mut peer) = data.peers.get_mut(tok) {
            match peer.connection {
                Connection::WebSock(_) => return,
                Connection::Timeout(_) => {}
            }
            let range = Range::new(5, 150);
            let mut rng = thread_rng();
            if let Ok(conn) = WebSocket::connect(peer.addr) {
                match conn.register(tok, ctx.eloop) {
                    Ok(_) => {
                        peer.connection = Connection::WebSock(conn);
                    } else {
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
}

fn try_io(&mut self, tok: Token, ev: EventSet, ctx: &mut Context)
        -> bool
{
    if let Some(dataref) = ctx.stats.read().unwrap().peers.clone() {
        let mut data = dataref.write().unwrap();
        if let Some(ref mut peer) = data.peers.get_mut(tok) {
            if ev.is_readable()
            return true;
        }
    }
    return false;
}
