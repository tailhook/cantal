use std::io;
use std::io::{Read, Write};
use std::net::{SocketAddr, SocketAddrV4};
use std::sync::{Arc, RwLock};
use std::default::Default;
use std::collections::{HashMap};

use mio::{EventLoop, Token, Handler, EventSet, PollOpt};
use mio::buf::ByteBuf;
use mio::{Sender, udp};
use nix::unistd::gethostname;
use cbor::{Decoder};
use rustc_serialize::Decodable;

use super::error::Error;
use super::stats::Stats;
use self::peer::{Peer};
use super::server;

mod peer;
mod gossip;


const GOSSIP: Token = Token(0);



fn hostname() -> String {
    let mut buf = [0u8; 256];
    gethostname(&mut buf).unwrap();
    for (idx, &ch) in buf.iter().enumerate() {
        if ch == 0 {
            return String::from_utf8(buf[..idx].to_owned()).unwrap();
        }
    }
    panic!("Bad hostname");
}


pub fn p2p_init(host: &str, port: u16)
    -> Result<Init, Error>
{
    let server = try!(udp::UdpSocket::bound(&SocketAddr::V4(
        SocketAddrV4::new(try!(host.parse()), port))));
    let mut eloop = try!(EventLoop::new());
    try!(eloop.register_opt(&server, GOSSIP,
        EventSet::readable(), PollOpt::level()));
    try!(eloop.timeout_ms(Timer::GossipBroadcast, gossip::INTERVAL));
    Ok(Init {
        sock: server,
        hostname: hostname(),
        channel: eloop.channel(),
        eloop: eloop,
    })
}

pub fn p2p_loop(init: Init, stats: &RwLock<Stats>,
    server_msg: Sender<server::Message>)
    -> Result<(), io::Error>
{
    let mut eloop = init.eloop;
    let stats = stats.read().unwrap().gossip.clone();
    eloop.run(&mut Context {
        stats: stats,
        queue: Default::default(),
        sock: init.sock,
        hostname: init.hostname,
        server_msg: server_msg,
    })
}


#[derive(Debug)]
pub enum Command {
    AddGossipHost(SocketAddr),
}

#[derive(Debug)]
pub enum Timer {
    GossipBroadcast,
}

pub struct Init {
    sock: udp::UdpSocket,
    hostname: String,
    eloop: EventLoop<Context>,
    pub channel: Sender<Command>,
}

struct Context {
    sock: udp::UdpSocket,
    stats: Arc<RwLock<GossipStats>>,
    queue: Vec<SocketAddr>,
    hostname: String,
    server_msg: Sender<server::Message>,
}

#[derive(Default)]
pub struct GossipStats {
    pub peers: HashMap<SocketAddr, Peer>,
}

impl Handler for Context {
    type Timeout = Timer;
    type Message = Command;

    fn ready(&mut self, _eloop: &mut EventLoop<Context>, tok: Token,
        _ev: EventSet)
    {
        match tok {
            GOSSIP => {
                let mut buf = ByteBuf::mut_with_capacity(4096);
                if let Ok(Some(addr)) = self.sock.recv_from(&mut buf) {
                    let mut dec = Decoder::from_reader(buf.flip());
                    match dec.decode::<gossip::Packet>().next() {
                        Some(Ok(packet)) => {
                            trace!("Packet {:?} from {:?}", packet, addr);
                            self.consume_gossip(packet, addr);
                        }
                        None => {
                            debug!("Empty packet from {:?}", addr);
                        }
                        Some(Err(e)) => {
                            debug!("Errorneous packet from {:?}: {}", addr, e);
                        }
                    }
                }
            }
            _ => unreachable!(),
        }
    }

    fn notify(&mut self, _eloop: &mut EventLoop<Context>, msg: Command) {
        use self::Command::*;
        trace!("Command {:?}", msg);
        match msg {
            AddGossipHost(ip) => {
                let ref mut peers = &mut self.stats.write().unwrap().peers;
                self.send_gossip(ip, peers);
            }
        }
    }

    fn timeout(&mut self, eloop: &mut EventLoop<Context>, msg: Timer) {
        match msg {
            Timer::GossipBroadcast => {
                self.gossip_broadcast();
                eloop.timeout_ms(Timer::GossipBroadcast,
                                 gossip::INTERVAL).unwrap();
            }
        }
    }
}
