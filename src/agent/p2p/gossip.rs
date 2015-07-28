use std::net::{SocketAddr};
use std::collections::HashMap;

use mio::Sender;
use time::{Timespec, Duration, get_time};
use rand::{thread_rng, Rng, sample};
use cbor::Encoder;
use mio::buf::ByteBuf;

use super::Context;
use super::peer::{Peer, Report};
use super::super::server::Message::NewHost;
use super::GossipStats;
use super::super::deps::{LockedDeps};


pub const INTERVAL: u64 = 1000;
pub const MIN_PROBE: u64 = 5000;  // Don't probe more often than 5 sec
pub const NUM_FRIENDS: u64 = 10;

#[derive(Debug, Clone, RustcEncodable, RustcDecodable)]
pub enum Packet {
    Ping {
        me: MyInfo,
        now: Timespec,
        friends: Vec<FriendInfo>,
    },
    Pong {
        me: MyInfo,
        ping_time: Timespec,
        peer_time: Timespec,
        friends: Vec<FriendInfo>,
    },
}

#[derive(Debug, Clone, RustcEncodable, RustcDecodable)]
pub struct MyInfo {
    host: String,
    peers: u32,
}

#[derive(Debug, Clone, RustcEncodable, RustcDecodable)]
pub struct FriendInfo {
    addr: String,
    host: Option<String>,
    peers: Option<u32>,
    last_report: Option<Timespec>,
    roundtrip: Option<(Timespec, u64)>,
}


fn after(tm: Option<Timespec>, target_time: Timespec) -> bool {
    return tm.map(|x| x >= target_time).unwrap_or(false);
}

fn get_friends_for(peers: &HashMap<SocketAddr, Peer>, peer: SocketAddr)
    -> Vec<FriendInfo>
{
    let mut rng = thread_rng();
    let other_peers = peers.iter().filter(|&(ref a, _)| **a != peer);
    let friends = sample(&mut rng, other_peers, 10);
    friends.into_iter().map(|(a, f)| FriendInfo {
        addr: a.to_string(),
        host: f.host.clone(),
        peers: f.report.as_ref().map(|x| x.peers),
        last_report: f.last_report,
        roundtrip: f.last_roundtrip,
    }).collect()
}

impl Context {
    pub fn gossip_broadcast(&mut self) {
        let cut_time = get_time() - Duration::milliseconds(MIN_PROBE as i64);
        let mut stats = self.deps.write::<GossipStats>();
        if self.queue.len() == 0 {
            if stats.peers.len() == 0 {
                return // nothing to do
            }
            self.queue = stats.peers.keys().cloned().collect();
        }
        thread_rng().shuffle(&mut self.queue[..]);
        for _ in 0..NUM_FRIENDS {
            if self.queue.len() == 0 {
                break;
            }
            let target_ip = self.queue.pop().unwrap();
            // if not expired yet
            if let Some(peer) = stats.peers.get(&target_ip) {
                if after(peer.last_probe, cut_time) ||
                   after(peer.last_report, cut_time) {
                    continue;  // don't probe too often
                }
            }
            self.send_gossip(target_ip, &mut stats.peers);
        }
    }
    fn apply_friends(&self, peers: &mut HashMap<SocketAddr, Peer>,
                     friends: Vec<FriendInfo>, source: SocketAddr)
    {
        for friend in friends {
            let addr: SocketAddr = if let Ok(val) = friend.addr.parse() {
                val
            } else {
                continue;
            };
            let peer = peers.entry(addr)
                .or_insert_with(|| {
                    self.deps.get::<Sender<_>>().unwrap()
                        .send(NewHost(addr))
                        .map_err(|_| error!("Error sending NewHost msg"))
                        .ok();
                    Peer::new(addr)
                });
            if peer.report.is_none() {
                peer.report = friend.peers.map(|x| Report {
                    peers: x,
                });
            }
            if peer.host != friend.host {
                if friend.host.is_some() && peer.host.is_some() {
                    debug!("Peer host is different for {} \
                            known {:?}, received {:?}",
                            addr, peer.host.as_ref().unwrap(),
                                  friend.host.as_ref().unwrap());
                } else if friend.host.is_some() {
                    peer.host = friend.host;
                }
            }
            if friend.roundtrip.is_some() {
                peer.random_peer_roundtrip = friend.roundtrip
                    .map(|(tm, rtt)| (source, tm, rtt));
            }
        }
    }
    pub fn send_gossip(&self, addr: SocketAddr,
                       peers: &mut HashMap<SocketAddr, Peer>)
    {
        debug!("Sending gossip to {}", addr);
        let mut buf = ByteBuf::mut_with_capacity(1024);
        {
            let mut e = Encoder::from_writer(&mut buf);
            e.encode(&[&Packet::Ping {
                me: MyInfo {
                    host: self.hostname.clone(),
                    peers: peers.len() as u32,
                },
                now: get_time(),
                friends: get_friends_for(peers, addr),
            }]).unwrap();
        }
        match self.sock.send_to(&mut buf.flip(), &addr) {
            Ok(_) => {
                let peer = peers.entry(addr)
                    .or_insert_with(|| {
                        self.deps.get::<Sender<_>>().unwrap()
                            .send(NewHost(addr))
                            .map_err(|_| error!("Error sending NewHost msg"))
                            .ok();
                        Peer::new(addr)
                    });
                peer.last_probe = Some(get_time());
            }
            Err(e) => {
                error!("Error sending probe to {:?}: {}", addr, e);
            }
        }
    }

    pub fn consume_gossip(&self, packet: Packet, addr: SocketAddr) {
        let tm = get_time();
        let mut stats = self.deps.write::<GossipStats>();
        match packet {
            Packet::Ping { me: info, now, friends } => {
                {
                    let peer = stats.peers.entry(addr)
                        .or_insert_with(|| {
                            self.deps.get::<Sender<_>>().unwrap()
                                .send(NewHost(addr))
                                .map_err(|_| error!(
                                    "Error sending NewHost msg"))
                                .ok();
                            Peer::new(addr)
                        });
                    peer.report = Some(Report {
                        peers: info.peers,
                    });
                    peer.host = Some(info.host.clone());
                    peer.last_report = Some(tm);
                }
                self.apply_friends(&mut stats.peers, friends, addr);
                let mut buf = ByteBuf::mut_with_capacity(1024);
                {
                    let mut e = Encoder::from_writer(&mut buf);
                    e.encode(&[&Packet::Pong {
                        me: MyInfo {
                            host: self.hostname.clone(),
                            peers: stats.peers.len() as u32,
                        },
                        ping_time: now,
                        peer_time: tm,
                        friends: get_friends_for(&stats.peers, addr),
                    }]).unwrap();
                }
                self.sock.send_to(&mut buf.flip(), &addr)
                    .map_err(|e| error!("Error sending probe to {:?}: {}",
                        addr, e))
                    .ok();
            }
            Packet::Pong { me: info, ping_time, peer_time, friends } => {
                {
                    let peer = stats.peers.entry(addr)
                        .or_insert_with(|| {
                            self.deps.get::<Sender<_>>().unwrap()
                                .send(NewHost(addr))
                                .map_err(|_| error!(
                                    "Error sending NewHost msg"))
                                .ok();
                            Peer::new(addr)
                        });
                    peer.report = Some(Report {
                        peers: info.peers,
                    });
                    // sanity check
                    if ping_time < tm && ping_time < peer_time {
                        peer.last_roundtrip = Some(
                            (tm, (tm - ping_time).num_milliseconds() as u64));
                    }
                    peer.host = Some(info.host.clone());
                    peer.last_report = Some(tm);
                }
                self.apply_friends(&mut stats.peers, friends, addr);
            }
        }
    }
}
