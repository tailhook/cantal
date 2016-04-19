use std::io::Write;
use std::net::{SocketAddr, SocketAddrV4};
use std::mem::replace;
use std::sync::Arc;
use std::collections::HashMap;
use std::os::unix::io::AsRawFd;

use mio::Sender;
use rand::{thread_rng, Rng, sample};
use cbor::Encoder;
use rustc_serialize::hex::ToHex;
use rustc_serialize::json::Json;

use storage::Storage;
use super::Context;
use super::peer::{Peer, Report};
use super::super::server::Message::Touch;
use super::GossipStats;
use super::super::deps::{LockedDeps};
use super::super::scan::time_ms;
use {HostId};


/// Wake up once per 1000 ms to send few probes
pub const INTERVAL: u64 = 1000;

/// Number of probes to send at each interval
pub const NUM_PROBES: u64 = 10;

/// If we got any probe or report during 5 seconds, don't probe this node
pub const MIN_PROBE: u64 = 5000;

/// But if we sent no probe within 60 seconds (but were receiving reports, so
/// didn't hit 5 seconds timeout above), we should send probe anyway. This
/// allows too keep roundtrip times on both nodes reasonably up to date
pub const MAX_PROBE: u64 = 60000;

/// Num of friend nodes to send within each request, everything must fit
/// MAX_PACKET_SIZE which is capped at maximum UDP packet size (65535),
/// better if it fits single IP packet (< 1500)
pub const NUM_FRIENDS: usize = 10;

/// After we had no reports from node for 20 seconds (but we sent probe during
/// this time) we consider node to be inaccessible by it's primary IP and are
/// trying to reach it by pinging any other random IP address.
pub const PREFAIL_TIME: u64 = 20_000;


/// Maximum expected roundtrip time. We consider report failing if it's not
/// received during this time. Note, this doesn't need to be absolute ceiling
/// of RTT, and we don't do any crazy things based on the timeout, this is
/// just heuristic for pre-fail condition.
pub const MAX_ROUNDTRIP: u64 = 2000;

/// After this time we consider node failing and don't send it in friendlist.
/// Note that all nodes that where up until we marked node as failinig do know
/// the node, and do ping it. This is currently used only
pub const FAIL_TIME: u64 = 3600_000;


/// This is time after last heartbeat when node will be removed from the list
/// of known nodes. This should be long after FAIL_TIME. (But not necessarily
/// 48x longer, as we do now).
/// Also note that node will be removed from all peers after
/// FAIL_TIME + REMOVE_TIME + longest-round-trip-time
pub const REMOVE_TIME: u64 = 2 * 86400_000;

/// This is a size of our UDP buffers. The maximum value depends on NUM_FRIENDS
/// and the number of IP addresses at each node. It's always capped at maximum
/// UDP packet size of 65535
pub const MAX_PACKET_SIZE: usize = 8192;



// Expectations:
//     MAX_PROBE > MIN_PROBE
//     MAX_ROUNDTRIP <= MAX_PROBE
//     FAIL_TIME + some big value < REMOVE_TIME

#[derive(Debug, Clone, RustcEncodable, RustcDecodable)]
pub enum Packet {
    Ping {
        cluster: String,
        me: MyInfo,
        now: u64,
        friends: Vec<FriendInfo>,
    },
    Pong {
        cluster: String,
        me: MyInfo,
        ping_time: u64,
        peer_time: u64,
        friends: Vec<FriendInfo>,
    },
}

#[derive(Debug, Clone, RustcEncodable, RustcDecodable)]
pub struct MyInfo {
    id: HostId,
    addresses: Vec<String>,
    host: String,
    name: String,
    report: Report,
}

#[derive(Debug, Clone, RustcEncodable, RustcDecodable)]
pub struct FriendInfo {
    id: HostId,
    my_primary_addr: Option<String>,
    addresses: Vec<String>,
    host: Option<String>,
    name: Option<String>,
    report: Option<(u64, Report)>,
    roundtrip: Option<(u64, u64)>,
}

fn get_friends(peers: &HashMap<HostId, Peer>, exclude: SocketAddr)
    -> Vec<FriendInfo>
{
    let mut rng = thread_rng();
    let other_peers = peers.values()
        .filter(|peer| !peer.addresses.contains(&exclude))
        .filter(|peer| !peer.is_failing());
    let friends = sample(&mut rng, other_peers, NUM_FRIENDS);
    friends.into_iter().map(|f| FriendInfo {
        id: f.id.clone(),
        my_primary_addr: f.primary_addr.map(|x| format!("{}", x)),
        addresses: f.addresses.iter().map(|x| format!("{}", x)).collect(),
        host: f.host.clone(),
        name: f.name.clone(),
        report: f.report.clone(),
        roundtrip: f.last_roundtrip.map(|(_, ts, rtt)| (ts, rtt)),
    }).collect()
}

impl Context {
    pub fn gossip_broadcast(&mut self) {
        let tm = time_ms();
        let cut_time = tm - MIN_PROBE;
        let mut stats = self.deps.write::<GossipStats>();
        if self.queue.len() == 0 {
            if stats.peers.len() == 0 {
                return // nothing to do
            }
            self.queue = stats.peers.keys().cloned().collect();
        }
        thread_rng().shuffle(&mut self.queue[..]);
        for _ in 0..NUM_PROBES {
            if self.queue.len() == 0 {
                break;
            }
            let id = self.queue.pop().unwrap();
            // if not expired yet
            let addr = stats.peers.get_mut(&id).and_then(|peer| {
                if !peer.has_fresh_report() {
                    let mut addr = peer.primary_addr;
                    if addr.is_none() || !peer.ping_primary_address() {
                        addr = peer.random_ping_addr()
                    };
                    addr.map(|addr| {
                        peer.last_probe = Some((tm, addr));
                        peer.probes_sent += 1;
                        addr
                    })
                } else {
                    None
                }
            });
            addr.map(|addr| {
                self.send_gossip(addr, &stats);
            });
        }
    }
    fn send_touch(&self, id: HostId) {
        self.deps.get::<Sender<_>>().unwrap()
            .send(Touch(id))
            .map_err(|_| error!("Error sending Touch msg"))
            .ok();
    }
    fn apply_friends(&self, stats: &mut GossipStats,
                     friends: Vec<FriendInfo>, source: SocketAddr)
    {
        for friend in friends.into_iter() {
            let sendto_addr = {
                let id = friend.id;
                let peer = stats.peers.entry(id.clone())
                    .or_insert_with(|| Peer::new(id.clone()));
                peer.apply_addresses(
                    // TODO(tailhook) filter out own IP addressses
                    friend.addresses.iter().filter_map(|x| x.parse().ok()),
                    false);
                peer.apply_report(friend.report, false);
                peer.apply_hostname(friend.host, false);
                peer.apply_node_name(friend.name, false);
                friend.roundtrip.map(|rtt|
                    peer.apply_roundtrip(rtt, source, false));
                if peer.primary_addr.is_none() {
                    let addr = friend.my_primary_addr.and_then(|x| {
                        x.parse().map_err(|e| error!("Can't parse IP address"))
                        .ok()
                    });
                    peer.primary_addr = addr;
                    addr.map(|addr| {
                        self.send_touch(id);
                        peer.last_probe = Some((time_ms(), addr));
                        peer.probes_sent += 1;
                        addr
                    });
                    addr
                } else {
                    None
                }
            };
            sendto_addr.map(|addr| {
                self.send_gossip(addr, stats);
            });
        }
    }
    pub fn send_gossip(&self, addr: SocketAddr, stats: &GossipStats)
    {
        let cluster = if let Some(ref name) = self.cluster_name { name } else {
            debug!("Skipping gossip {}, no cluster name", addr);
            return;
        };
        debug!("Sending gossip {}", addr);
        let mut buf = Vec::with_capacity(MAX_PACKET_SIZE);
        {
            let mut e = Encoder::from_writer(&mut buf);
            e.encode(&[&Packet::Ping {
                cluster: cluster.clone(),
                me: MyInfo {
                    id: self.machine_id.clone(),
                    addresses: self.addresses.iter()
                        .map(|x| x.to_string()).collect(),
                    host: self.hostname.clone(),
                    name: self.name.clone(),
                    report: Report {
                        peers: stats.peers.len() as u32,
                        has_remote: stats.has_remote,
                    },
                },
                now: time_ms(),
                friends: get_friends(&stats.peers, addr),
            }]).unwrap();
        }
        if buf.len() >= MAX_PACKET_SIZE {
            // Unfortunately cbor encoder doesn't report error of truncated
            // data so we consider full buffer the truncated data
            error!("Error sending probe to {}: Data is too long. \
                All limits are compile-time. So this error basically means \
                cantal developers were unwise at choosing the right values. \
                If you didn't tweak the limits yourself, please file an issue \
                at http://github.com/tailhook/cantal/issues", addr);
        }
        if let Err(e) = self.sock.send_to(&buf[..], &addr) {
            error!("Error sending probe to {}: {}", addr, e);
        }
    }

    pub fn consume_gossip(&self, packet: Packet, addr: SocketAddr,
        stats: &mut GossipStats) {
        let tm = time_ms();
        let v4: SocketAddrV4 =
            if let SocketAddr::V4(val) = addr {
                val
            } else {
                return;
            };

        match packet {
            Packet::Ping { cluster,  me: info, now, friends } => {
                {
                    if Some(&cluster) != self.cluster_name.as_ref() {
                        info!("Got packet from cluster {:?}", cluster);
                        return;
                    }
                    let id = info.id.clone();
                    let peer = stats.peers.entry(id.clone())
                        .or_insert_with(|| Peer::new(id.clone()));
                    peer.apply_addresses(
                        // TODO(tailhook) filter out own IP addressses
                        info.addresses.iter().filter_map(|x| x.parse().ok()),
                        true);
                    peer.apply_report(Some((tm, info.report)), true);
                    peer.apply_hostname(Some(info.host), true);
                    peer.apply_node_name(Some(info.name), true);
                    peer.pings_received += 1;
                    if peer.primary_addr.as_ref() != Some(&addr) {
                        peer.primary_addr = Some(addr);
                        self.send_touch(id);
                    }
                }
                self.apply_friends(&mut *stats, friends, addr);
                let mut buf = Vec::with_capacity(MAX_PACKET_SIZE);
                {
                    let mut e = Encoder::from_writer(&mut buf);
                    e.encode(&[&Packet::Pong {
                        cluster: cluster,
                        me: MyInfo {
                            id: self.machine_id.clone(),
                            addresses: self.addresses.iter()
                                .map(|x| x.to_string()).collect(),
                            host: self.hostname.clone(),
                            name: self.name.clone(),
                            report: Report {
                                peers: stats.peers.len() as u32,
                                has_remote: stats.has_remote,
                            },
                        },
                        ping_time: now,
                        peer_time: tm,
                        friends: get_friends(&stats.peers, addr),
                    }]).unwrap();
                }

                if buf.len() == MAX_PACKET_SIZE {
                    // Unfortunately cbor encoder doesn't report error of truncated
                    // data so we consider full buffer the truncated data
                    error!("Error sending probe to {}: Data is too long. \
                        All limits are compile-time. So this error basically \
                        means  cantal developers were unwise at choosing the \
                        right values. If you didn't tweak the limits \
                        yourself, please file an issue at \
                        http://github.com/tailhook/cantal/issues", addr);
                }
                self.sock.send_to(&buf[..], &addr)
                    .map_err(|e| error!("Error sending probe to {:?}: {}",
                        addr, e))
                    .ok();
            }
            Packet::Pong { cluster, me: info, ping_time, peer_time, friends }
            => {
                {
                    if Some(&cluster) != self.cluster_name.as_ref() {
                        info!("Got packet from cluster {:?}", cluster);
                        return;
                    }
                    let id = info.id.clone();
                    let peer = stats.peers.entry(id.clone())
                        .or_insert_with(|| Peer::new(id.clone()));
                    peer.apply_addresses(
                        // TODO(tailhook) filter out own IP addressses
                        info.addresses.iter().filter_map(|x| x.parse().ok()),
                        true);
                    peer.apply_report(Some((tm, info.report)), true);
                    peer.pongs_received += 1;
                    // sanity check
                    if ping_time <= tm && ping_time <= peer_time {
                        peer.apply_roundtrip((tm, (tm - ping_time)),
                            addr, true);
                    }
                    peer.apply_hostname(Some(info.host), true);
                    peer.apply_node_name(Some(info.name), true);
                    if peer.primary_addr.as_ref() != Some(&addr) {
                        peer.primary_addr = Some(addr);
                        self.send_touch(id);
                    }
                }
                self.apply_friends(&mut *stats, friends, addr);
            }
        }
    }

    pub fn remove_failed_nodes(&mut self) {
        let mut statsguard = self.deps.write::<GossipStats>();
        let ref mut stats = &mut *statsguard;
        stats.peers = replace(&mut stats.peers, HashMap::new()).into_iter()
            .filter(|&(ref id, ref peer)| {
                if peer.should_remove() {
                    warn!("Peer {} / {:?} is removed",
                        id.to_hex(), peer.addresses);
                    false
                } else {
                    true
                }
            }).collect();
    }

    pub fn store_peers(&mut self) {
        let data = {
            let mut buf = Vec::with_capacity(1024);
            let mut statsguard = self.deps.write::<GossipStats>();
            let ref mut stats = &mut *statsguard;
            let addrs = stats.peers.iter()
                .flat_map(|(id, peer)| peer.addresses.iter())
                .map(|x| Json::String(x.to_string()))
                .collect();
            write!(&mut buf, "{}", Json::Object(vec![
                (String::from("ip_addresses"), Json::Array(addrs)),
                ].into_iter().collect())).unwrap();
            buf.into_boxed_slice()
        };
        self.deps.get::<Arc<Storage>>().map(|storage| {
            storage.store_peers(data);
        });
    }
}
