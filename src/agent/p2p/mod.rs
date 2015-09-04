use std::io;
use std::io::{Write};
use std::net::{SocketAddr, SocketAddrV4};
use std::sync::{Arc, RwLock};
use std::default::Default;
use std::collections::{HashMap};

use mio::{EventLoop, Token, Handler, EventSet, PollOpt};
use mio::buf::ByteBuf;
use mio::{udp};
use cbor::{Decoder};
use rustc_serialize::Decodable;

use super::error::Error;
use self::peer::{Peer};
use super::deps::{Dependencies, LockedDeps};
use {HostId};

mod peer;
mod gossip;


const GOSSIP: Token = Token(0);
const GARBAGE_COLLECTOR_INTERVAL: u64 = 300_000; // 5 min



pub fn p2p_init(deps: &mut Dependencies, host: &str, port: u16,
    machine_id: Vec<u8>, addresses: Vec<SocketAddr>,
    hostname: String, name: String)
    -> Result<Init, Error>
{
    let server = try!(udp::UdpSocket::bound(&SocketAddr::V4(
        SocketAddrV4::new(try!(host.parse()), port))));
    let mut eloop = try!(EventLoop::new());
    try!(eloop.register_opt(&server, GOSSIP,
        EventSet::readable(), PollOpt::level()));
    try!(eloop.timeout_ms(Timer::GossipBroadcast, gossip::INTERVAL));
    try!(eloop.timeout_ms(Timer::GarbageCollector,
        GARBAGE_COLLECTOR_INTERVAL));

    deps.insert(eloop.channel());
    deps.insert(Arc::new(RwLock::new(GossipStats::default())));

    Ok(Init {
        sock: server,
        machine_id: machine_id,
        addresses: addresses,
        hostname: hostname,
        name: name,
        eloop: eloop,
    })
}

pub fn p2p_loop(init: Init, deps: Dependencies)
    -> Result<(), io::Error>
{
    let mut eloop = init.eloop;
    eloop.run(&mut Context {
        queue: Default::default(),
        sock: init.sock,
        machine_id: init.machine_id,
        addresses: init.addresses,
        hostname: init.hostname,
        name: init.name,
        deps: deps,
    })
}


#[derive(Debug)]
pub enum Command {
    AddGossipHost(SocketAddr),
    RemoteSwitch(bool),
}

#[derive(Debug)]
pub enum Timer {
    GossipBroadcast,
    GarbageCollector,
}

pub struct Init {
    sock: udp::UdpSocket,
    machine_id: Vec<u8>,
    addresses: Vec<SocketAddr>,
    hostname: String,
    name: String,
    eloop: EventLoop<Context>,
}

struct Context {
    sock: udp::UdpSocket,
    queue: Vec<HostId>,
    machine_id: HostId,
    addresses: Vec<SocketAddr>,
    hostname: String,
    name: String,
    deps: Dependencies,
}

#[derive(Default)]
pub struct GossipStats {
    pub peers: HashMap<HostId, Peer>,
    pub has_remote: bool,
}

impl Handler for Context {
    type Timeout = Timer;
    type Message = Command;

    fn ready(&mut self, _eloop: &mut EventLoop<Context>, tok: Token,
        _ev: EventSet)
    {
        match tok {
            GOSSIP => {
                let mut stats = self.deps.write::<GossipStats>();
                loop {
                    let mut buf = ByteBuf::mut_with_capacity(8192);
                    if let Ok(Some(addr)) = self.sock.recv_from(&mut buf) {
                        let mut dec = Decoder::from_reader(buf.flip());
                        match dec.decode::<gossip::Packet>().next() {
                            Some(Ok(packet)) => {
                                trace!("Packet {:?} from {:?}", packet, addr);
                                self.consume_gossip(packet, addr, &mut *stats);
                            }
                            None => {
                                debug!("Empty packet from {:?}", addr);
                            }
                            Some(Err(e)) => {
                                debug!("Errorneous packet from {:?}: {}", addr, e);
                            }
                        }
                    } else {
                        break;
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
                let ref mut stats = self.deps.write::<GossipStats>();
                self.send_gossip(ip, stats);
            }
            RemoteSwitch(val) => {
                let ref mut stats = self.deps.write::<GossipStats>();
                stats.has_remote = val;
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
            Timer::GarbageCollector => {
                self.remove_failed_nodes();
                eloop.timeout_ms(Timer::GarbageCollector,
                                 GARBAGE_COLLECTOR_INTERVAL).unwrap();
            }
        }
    }
}
