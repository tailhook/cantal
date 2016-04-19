use std::io;
use std::io::{Write, Cursor};
use std::net::{SocketAddr, SocketAddrV4};
use std::sync::{Arc, RwLock};
use std::default::Default;
use std::collections::{HashMap};

use mio::{EventLoop, Token, Handler, EventSet, PollOpt};
use mio::{udp};
use cbor::{Decoder};
use rustc_serialize::Decodable;
use rand::{thread_rng};
use rand::distributions::{IndependentSample, Range};

use super::error::Error;
use self::peer::{Peer};
use super::deps::{Dependencies, LockedDeps};
use {HostId};
use self::gossip::MAX_PACKET_SIZE;

mod peer;
mod gossip;


const GOSSIP: Token = Token(0);
const GARBAGE_COLLECTOR_INTERVAL: u64 = 300_000; // 5 min
/// Sometimes packet lost (it's UDP) so we retry 5 times
const ADD_HOST_RETRY_TIMES: u32 = 5;
/// The retry interval, it will be randomized from 0.5x to 1.5x
/// Note: randomization is then rounded to tick size of 100ms by mio
const ADD_HOST_RETRY_INTERVAL: u64 = 1000;


pub fn p2p_init(deps: &mut Dependencies, host: &str, port: u16,
    machine_id: Vec<u8>, addresses: Vec<SocketAddr>,
    hostname: String, name: String, cluster_name: Option<String>)
    -> Result<Init, Error>
{
    let server = try!(udp::UdpSocket::bound(&SocketAddr::V4(
        SocketAddrV4::new(try!(host.parse()), port))));
    let mut eloop = try!(EventLoop::new());
    try!(eloop.register(&server, GOSSIP,
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
        cluster_name: cluster_name,
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
        cluster_name: init.cluster_name,
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
    AddHost(SocketAddr, u32),
}

pub struct Init {
    sock: udp::UdpSocket,
    machine_id: Vec<u8>,
    addresses: Vec<SocketAddr>,
    hostname: String,
    name: String,
    cluster_name: Option<String>, // TODO(tailhook) disable entirely if None
    eloop: EventLoop<Context>,
}

struct Context {
    sock: udp::UdpSocket,
    queue: Vec<HostId>,
    machine_id: HostId,
    addresses: Vec<SocketAddr>,
    hostname: String,
    name: String,
    cluster_name: Option<String>, // TODO(tailhook) disable entirely if None
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
                    let mut buf = [0u8; MAX_PACKET_SIZE];
                    if let Ok(Some((n, addr))) = self.sock.recv_from(&mut buf) {
                        let mut dec = Decoder::from_reader(&buf[..n]);
                        match dec.decode::<gossip::Packet>().next() {
                            Some(Ok(packet)) => {
                                trace!("Packet {:?} from {:?}", packet, addr);
                                self.consume_gossip(packet, addr, &mut *stats);
                            }
                            None => {
                                warn!("Empty or truncated packet from {:?}",
                                      addr);
                            }
                            Some(Err(e)) => {
                                warn!("Errorneous packet from {:?}: {}",
                                    addr, e);
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

    fn notify(&mut self, eloop: &mut EventLoop<Context>, msg: Command) {
        use self::Command::*;
        trace!("Command {:?}", msg);
        match msg {
            AddGossipHost(ip) => {
                let ref mut stats = self.deps.write::<GossipStats>();
                self.send_gossip(ip, stats);
                // Sometimes first packet fails, so we try few times
                let range = Range::new(ADD_HOST_RETRY_INTERVAL/2,
                    ADD_HOST_RETRY_INTERVAL*3/2);
                let mut rng = thread_rng();
                eloop.timeout_ms(Timer::AddHost(ip, ADD_HOST_RETRY_TIMES),
                                 range.ind_sample(&mut rng)).unwrap();
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
                self.store_peers();
                eloop.timeout_ms(Timer::GarbageCollector,
                                 GARBAGE_COLLECTOR_INTERVAL).unwrap();
            }
            Timer::AddHost(ip, num) => {
                let ref mut stats = self.deps.write::<GossipStats>();
                self.send_gossip(ip, stats);
                // Sometimes first packet fails, so we try few times
                if num > 0 {
                    let range = Range::new(ADD_HOST_RETRY_INTERVAL/2,
                        ADD_HOST_RETRY_INTERVAL*3/2);
                    let mut rng = thread_rng();
                    eloop.timeout_ms(Timer::AddHost(ip, num-1),
                                     range.ind_sample(&mut rng)).unwrap();
                }
            }
        }
    }
}
