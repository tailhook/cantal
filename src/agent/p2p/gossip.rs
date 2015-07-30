use std::net::{SocketAddr};
use std::collections::HashMap;

use mio::Sender;
use rand::{thread_rng, Rng, sample};
use cbor::Encoder;
use mio::buf::ByteBuf;

use super::Context;
use super::peer::{Peer, Report};
use super::super::server::Message::NewHost;
use super::GossipStats;
use super::super::deps::{LockedDeps};
use super::super::remote::Peers as RemotePeers;
use super::super::scan::time_ms;


pub const INTERVAL: u64 = 1000;
pub const MIN_PROBE: u64 = 5000;  // Don't probe more often than 5 sec
pub const NUM_FRIENDS: u64 = 10;

#[derive(Debug, Clone, RustcEncodable, RustcDecodable)]
pub enum Packet {
    Ping {
        me: MyInfo,
        now: u64,
        friends: Vec<FriendInfo>,
    },
    Pong {
        me: MyInfo,
        ping_time: u64,
        peer_time: u64,
        friends: Vec<FriendInfo>,
    },
}

#[derive(Debug, Clone, RustcEncodable, RustcDecodable)]
pub struct MyInfo {
    host: String,
    report: Report,
}

#[derive(Debug, Clone, RustcEncodable, RustcDecodable)]
pub struct FriendInfo {
    addr: String,
    host: Option<String>,
    report: Option<(u64, Report)>,
    roundtrip: Option<(u64, u64)>,
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
        report: f.report.clone(),
        roundtrip: f.last_roundtrip,
    }).collect()
}

fn apply_report(dest: &mut Option<(u64, Report)>, src: Option<(u64, Report)>) {
    if match (&dest, &src) {
        (&&mut Some((pts, _)), &Some((fts, _)))
        if pts < fts  // apply only newer report
        => true,
        (&&mut None, &Some((_, _)))  // or if one did not exists
        => true,
        _ => false
    } {
        *dest = src;
    }
}

impl Context {
    pub fn gossip_broadcast(&mut self) {
        let cut_time = time_ms() - MIN_PROBE;
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
                if peer.last_probe.map(|x| x > cut_time).unwrap_or(false) ||
                   peer.last_report_direct.map(|x| x > cut_time)
                    .unwrap_or(false)
                {
                    continue;  // don't probe too often
                }
            }
            self.send_gossip(target_ip, &mut stats.peers);
        }
    }
    fn apply_friends(&self, peers: &mut HashMap<SocketAddr, Peer>,
                     friends: Vec<FriendInfo>, source: SocketAddr)
    {
        for friend in friends.into_iter() {
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
            apply_report(&mut peer.report, friend.report);
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
        // This "has_remote" thing is put here but it risks deadlock
        // TODO(tailhook) fix this deadlock please!!!
        let has_remote = self.deps.read::<Option<RemotePeers>>().is_some();

        debug!("Sending gossip to {}", addr);
        let mut buf = ByteBuf::mut_with_capacity(1024);
        {
            let mut e = Encoder::from_writer(&mut buf);
            e.encode(&[&Packet::Ping {
                me: MyInfo {
                    host: self.hostname.clone(),
                    report: Report {
                        peers: peers.len() as u32,
                        has_remote: has_remote,
                    },
                },
                now: time_ms(),
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
                peer.last_probe = Some(time_ms());
            }
            Err(e) => {
                error!("Error sending probe to {:?}: {}", addr, e);
            }
        }
    }

    pub fn consume_gossip(&self, packet: Packet, addr: SocketAddr) {
        let tm = time_ms();

        // This "has_remote" thing is put here to reduce the chance of
        // deadlock (i.e. get remote peers before logging gossip stats
        // However, it's not always needed so, may be optimize it?
        let has_remote = self.deps.read::<Option<RemotePeers>>().is_some();

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
                    apply_report(&mut peer.report, Some((tm, info.report)));
                    peer.host = Some(info.host.clone());
                    peer.last_report_direct = Some(tm);
                }
                self.apply_friends(&mut stats.peers, friends, addr);
                let mut buf = ByteBuf::mut_with_capacity(1024);
                {
                    let mut e = Encoder::from_writer(&mut buf);
                    e.encode(&[&Packet::Pong {
                        me: MyInfo {
                            host: self.hostname.clone(),
                            report: Report {
                                peers: stats.peers.len() as u32,
                                has_remote: has_remote,
                            },
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
                    apply_report(&mut peer.report, Some((tm, info.report)));
                    // sanity check
                    if ping_time < tm && ping_time < peer_time {
                        peer.last_roundtrip = Some(
                            (tm, (tm - ping_time) as u64));
                    }
                    peer.host = Some(info.host.clone());
                    peer.last_report_direct = Some(tm);
                }
                self.apply_friends(&mut stats.peers, friends, addr);
            }
        }
    }
}
