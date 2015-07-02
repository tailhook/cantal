use std::io::{Read, Write};
use std::net::SocketAddr;
use std::sync::{Arc, RwLock, mpsc};
use std::default::Default;
use std::collections::{HashMap};

use mio;
use mio::{EventLoop, Token, NonBlock, ReadHint, Handler};
use mio::buf::ByteBuf;
use mio::{Sender, udp};
use nix::unistd::gethostname;
use cbor::{Decoder};
use rustc_serialize::Decodable;

use super::stats::Stats;
use self::peer::{Peer};

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


pub fn p2p_loop(stats: &RwLock<Stats>, host: &str, port: u16,
    sender: mpsc::Sender<mio::Sender<Command>>) {
    let server = udp::bind(&format!("{}:{}", host, port).parse().unwrap()
                            ).unwrap();
    let mut eloop = EventLoop::new().unwrap();
    eloop.register(&server, GOSSIP).unwrap();
    eloop.timeout_ms(Timer::GossipBroadcast, gossip::INTERVAL).unwrap();
    sender.send(eloop.channel()).unwrap();
    let mut ctx = Context {
        sock: server,
        stats: stats.read().unwrap().gossip.clone(),
        queue: Default::default(),
        hostname: hostname(),
    };
    eloop.run(&mut ctx).unwrap();
}


#[derive(Debug)]
pub enum Command {
    AddGossipHost(SocketAddr),
}

#[derive(Debug)]
pub enum Timer {
    GossipBroadcast,
}

struct Context {
    sock: NonBlock<udp::UdpSocket>,
    stats: Arc<RwLock<GossipStats>>,
    queue: Vec<SocketAddr>,
    hostname: String,
}

#[derive(Default)]
pub struct GossipStats {
    pub peers: HashMap<SocketAddr, Peer>,
}

impl Handler for Context {
    type Timeout = Timer;
    type Message = Command;

    fn readable(&mut self, _eloop: &mut EventLoop<Context>,
                tok: Token, _hint: ReadHint)
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
