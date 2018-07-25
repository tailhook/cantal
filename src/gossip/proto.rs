use std::cmp::{PartialOrd, Ordering, min};
use std::collections::{HashMap, BinaryHeap};
use std::io::Write;
use std::mem;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::{Instant, Duration};

use serde_cbor::from_slice;
use serde_cbor::ser::to_writer;
use futures::{Future, Async, Stream};
use quick_error::ResultExt;
use rand::{thread_rng, Rng};
use tk_easyloop::{self, timeout};
use tokio_core::net::UdpSocket;
use tokio_core::reactor::Timeout;
use void::{Void, unreachable};
use rustc_serialize::json::Json;

use gossip::command::Command;
use gossip::Config;
use gossip::errors::InitError;
use gossip::info::Info;
use gossip::peer::{Report, Peer};
use id::Id as HostId;
use storage::Storage;
use time_util::time_ms;
use libcantal::Integer;
use incoming::{self, Subscription};

lazy_static! {
    pub static ref NUM_PEERS: Integer = Integer::new();
    pub static ref NUM_STALE: Integer = Integer::new();
}

#[derive(Eq)]
struct FutureHost {
    deadline: Instant,
    address: SocketAddr,
    attempts: u32,
    timeout: Duration,
}

#[derive(Clone, Copy)]
enum AddrStatus {
    Available,
    PingSent,
}

pub struct Proto<S> {
    sock: UdpSocket,
    config: Arc<Config>,
    info: Arc<Mutex<Info>>,
    addr_status: HashMap<SocketAddr, AddrStatus>,
    add_queue: BinaryHeap<FutureHost>,
    ping_queue: Vec<HostId>,
    next_ping: Instant,
    next_gc: Instant,
    clock: Timeout,
    stream: S,
    // TODO(tailhook)
    // remote: Remote,
    storage: Arc<Storage>,
    input_buf: Vec<u8>,
    output_buf: Vec<u8>,
    incoming: incoming::channel::Sender,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Packet {
    Ping {
        cluster: Arc<String>,
        me: MyInfo,
        now: u64,
        friends: Vec<FriendInfo>,
    },
    Pong {
        cluster: Arc<String>,
        me: MyInfo,
        ping_time: u64,
        peer_time: u64,
        friends: Vec<FriendInfo>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MyInfo {
    id: HostId,
    addresses: Arc<Vec<String>>,
    host: Arc<String>,
    name: Arc<String>,
    report: Report,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FriendInfo {
    pub id: HostId,
    pub my_primary_addr: Option<String>,
    pub addresses: Vec<String>,
    pub host: Option<String>,
    pub name: Option<String>,
    pub report: Option<(u64, Report)>,
    pub roundtrip: Option<(u64, u64)>,
}


impl<S: Stream<Item=Command, Error=Void>> Proto<S> {
    pub fn new(info: &Arc<Mutex<Info>>, config: &Arc<Config>, stream: S,
        storage: &Arc<Storage>, incoming: &incoming::channel::Sender)
       -> Result<Proto<S>, InitError>
    {
        let s = UdpSocket::bind(&config.bind, &tk_easyloop::handle())
            .context(config.bind)?;
        Ok(Proto {
            sock: s,
            storage: storage.clone(),
            config: config.clone(),
            info: info.clone(),
            stream: stream,
            addr_status: HashMap::new(),
            add_queue: BinaryHeap::new(),
            ping_queue: Vec::new(),
            next_ping: Instant::now() + config.interval,
            next_gc: Instant::now() + config.garbage_collector_interval,
            clock: timeout(config.interval),
            input_buf: vec![0; config.max_packet_size],
            output_buf: vec![0; config.max_packet_size],
            incoming: incoming.clone(),
        })
    }
}

impl<S: Stream<Item=Command, Error=Void>> Future for Proto<S> {
    type Item = ();
    type Error = ();
    fn poll(&mut self) -> Result<Async<()>, ()> {
        let current_timeout = self.next_wakeup();
        loop {
            self.internal_messages().unwrap_or_else(|e| unreachable(e));
            self.receive_messages();
            if Instant::now() >= self.next_gc {
                self.garbage_collector();
                self.next_gc = Instant::now() +
                    self.config.garbage_collector_interval;
            }
            self.retry_new_hosts();
            if Instant::now() >= self.next_ping {
                self.ping_hosts();
                self.next_ping = Instant::now() + self.config.interval;
            }

            let new_timeout = self.next_wakeup();
            if new_timeout != current_timeout {
                let now = Instant::now();
                if new_timeout <= now {
                    continue;
                } else {
                    let mut timeo = timeout(new_timeout.duration_since(now));
                    // We need to `poll` it to get wakeup scheduled
                    match timeo.poll().map_err(|_| ())? {
                        Async::Ready(()) => continue,
                        Async::NotReady => {}
                    }
                    self.clock = timeo;
                    break;
                }
            } else {
                match self.clock.poll().map_err(|_| ())? {
                    Async::Ready(()) => continue,
                    Async::NotReady => break,
                }
            }
        }
        Ok(Async::NotReady)
    }
}

impl<S: Stream<Item=Command, Error=Void>> Proto<S> {
    fn next_wakeup(&self) -> Instant {
        let static_next = min(self.next_ping, self.next_gc);
        self.add_queue.peek().map(|x| min(x.deadline, static_next))
            .unwrap_or(static_next)
    }
    fn internal_messages(&mut self) -> Result<(), Void> {
        use gossip::command::Command::*;
        while let Async::Ready(msg) = self.stream.poll()? {
            let msg = msg.expect("gossip stream never ends");
            match msg {
                AddHost(addr) => {
                    use self::AddrStatus::*;
                    let status = self.addr_status.get(&addr).map(|x| *x);
                    match status {
                        Some(Available) => {}
                        Some(PingSent)|None => {
                            // We send ping anyway, so that you can trigger
                            // adding failed host faster (not waiting for
                            // longer exponential back-off at this moment).
                            //
                            // While at a glance this may make us susceptible
                            // to DoS attacks, but presumably this requires a
                            // lot less resources than the initial HTTP
                            // request or websocket message that triggers
                            // `AddHost()` message itself
                            self.send_gossip(addr);
                        }
                    }
                    match status {
                        Some(Available) => {}
                        // .. but we keep same timestamp for the next retry
                        // to avoid memory leaks
                        Some(PingSent) => { }
                        None => {
                            self.addr_status.insert(addr, PingSent);
                            let timeout = self.config.add_host_first_sleep();
                            self.add_queue.push(FutureHost {
                                deadline: Instant::now() + timeout,
                                address: addr,
                                attempts: 1,
                                timeout: timeout,
                            });
                        }
                    }
                }
            }
        }
        Ok(())
    }
    fn receive_messages(&mut self) {
        // Steal buffer to satisfy borrow checker
        // It should be cheap, as empty vector is non-allocating
        let mut buf = mem::replace(&mut self.input_buf, Vec::new());
        assert!(buf.len() == self.config.max_packet_size);

        while let Ok((bytes, addr)) = self.sock.recv_from(&mut buf) {
            match from_slice(&buf[..bytes]) {
                Ok(packet) => {
                    trace!("Packet {:?} from {:?}", packet, addr);
                    self.consume_gossip(packet, addr);
                }
                Err(e) => {
                    warn!("Errorneous packet from {:?}: {}",
                        addr, e);
                }
            }
        }
        // return buffer back
        self.input_buf = buf;
    }
    pub fn consume_gossip(&mut self, packet: Packet, addr: SocketAddr) {
        use self::AddrStatus::*;

        let tm = time_ms();
        let mut update = false;

        match packet {
            Packet::Ping { cluster,  me: pinfo, now, friends } => {
                {
                    if cluster != self.config.cluster_name {
                        info!("Got packet from cluster {:?}", cluster);
                        return;
                    }
                    if pinfo.id == self.config.machine_id {
                        debug!("Got packet from myself");
                        return;
                    }
                    let id = pinfo.id.clone();
                    let mut info = self.info.lock()
                        .expect("gossip info poisoned");
                    self.addr_status.insert(addr, Available);
                    let peer = info.peers.entry(id.clone())
                        .or_insert_with(|| {
                            update = true;
                            Arc::new(Peer::new(id.clone()))
                        });
                    let peer = Arc::make_mut(peer);
                    peer.apply_addresses(
                        // TODO(tailhook) filter out own IP addressses
                        pinfo.addresses.iter().filter_map(|x| x.parse().ok()),
                        true);
                    peer.apply_report(Some((tm, pinfo.report)), true);
                    peer.apply_hostname(Some(pinfo.host.as_ref()), true);
                    peer.apply_node_name(Some(pinfo.name.as_ref()), true);
                    peer.pings_received += 1;
                    if peer.primary_addr.as_ref() != Some(&addr) {
                        peer.primary_addr = Some(addr);
                        // TODO(remote)
                        // self.remote.touch(id);
                    }
                }
                self.apply_friends(friends, addr);
                let ref mut buf = &mut self.output_buf;
                buf.truncate(0);
                {
                    let info = self.info.lock().expect("gossip info poisoned");
                    to_writer(&mut *buf, &Packet::Pong {
                        cluster: cluster,
                        me: MyInfo {
                            id: self.config.machine_id.clone(),
                            addresses: self.config.str_addresses.clone(),
                            host: self.config.hostname.clone(),
                            name: self.config.name.clone(),
                            report: Report {
                                peers: info.peers.len() as u32,
                                has_remote: info.has_remote,
                            },
                        },
                        ping_time: now,
                        peer_time: tm,
                        friends: info.get_friends(addr, &self.config),
                    }).unwrap();
                }

                if buf.len() >= self.config.max_packet_size {
                    // Unfortunately cbor encoder doesn't report error of
                    // truncated data so we consider full buffer the truncated
                    // data
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
            Packet::Pong { cluster, me: pinfo, ping_time, peer_time, friends }
            => {
                {
                    if cluster != self.config.cluster_name {
                        info!("Got packet from cluster {:?}", cluster);
                        return;
                    }
                    if pinfo.id == self.config.machine_id {
                        debug!("Got packet from myself");
                        return;
                    }
                    let mut info = self.info.lock()
                        .expect("gossip info poisoned");
                    let id = pinfo.id.clone();
                    self.addr_status.insert(addr, Available);
                    let peer = info.peers.entry(id.clone())
                        .or_insert_with(|| {
                            update = true;
                            Arc::new(Peer::new(id.clone()))
                        });
                    let peer = Arc::make_mut(peer);
                    peer.apply_addresses(
                        // TODO(tailhook) filter out own IP addressses
                        pinfo.addresses.iter().filter_map(|x| x.parse().ok()),
                        true);
                    peer.apply_report(Some((tm, pinfo.report)), true);
                    peer.pongs_received += 1;
                    // sanity check
                    if ping_time <= tm && ping_time <= peer_time {
                        peer.apply_roundtrip((tm, (tm - ping_time)),
                            addr, true);
                    }
                    peer.apply_hostname(Some(pinfo.host.as_ref()), true);
                    peer.apply_node_name(Some(pinfo.name.as_ref()), true);
                    if peer.primary_addr.as_ref() != Some(&addr) {
                        peer.primary_addr = Some(addr);
                        // TODO(tailhook) remote
                        // self.remote.touch(id);
                    }
                }
                self.apply_friends(friends, addr);
            }
        }
        if update {
            self.update_metrics();
        }
        self.incoming.trigger(Subscription::Peers);
    }
    fn send_gossip(&mut self, addr: SocketAddr) {
        debug!("Sending gossip {}", addr);
        let ref mut buf = self.output_buf;
        buf.truncate(0);
        {
            let info = self.info.lock().expect("gossip info poisoned");
            to_writer(&mut *buf, &Packet::Ping {
                cluster: self.config.cluster_name.clone(),
                me: MyInfo {
                    id: self.config.machine_id.clone(),
                    addresses: self.config.str_addresses.clone(),
                    host: self.config.hostname.clone(),
                    name: self.config.name.clone(),
                    report: Report {
                        peers: info.peers.len() as u32,
                        has_remote: info.has_remote,
                    },
                },
                now: time_ms(),
                friends: info.get_friends(addr, &self.config),
            }).unwrap();
        }
        if buf.len() >= self.config.max_packet_size {
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
    fn apply_friends(&mut self, friends: Vec<FriendInfo>, source: SocketAddr) {
        for friend in friends.into_iter() {
            let sendto_addr = {
                let id = friend.id;
                if id == self.config.machine_id {
                    debug!("Got myself in friend list");
                    continue;
                }
                let mut info = self.info.lock()
                    .expect("gossip info poisoned");
                let peer = info.peers.entry(id.clone())
                    .or_insert_with(|| Arc::new(Peer::new(id.clone())));
                let peer = Arc::make_mut(peer);
                peer.apply_addresses(
                    // TODO(tailhook) filter out own IP addressses
                    friend.addresses.iter().filter_map(|x| x.parse().ok()),
                    false);
                peer.apply_report(friend.report, false);
                peer.apply_hostname(friend.host.as_ref().map(|x| &**x), false);
                peer.apply_node_name(
                    friend.name.as_ref().map(|x| &**x), false);
                friend.roundtrip.map(|rtt|
                    peer.apply_roundtrip(rtt, source, false));
                if peer.primary_addr.is_none() {
                    let addr = friend.my_primary_addr.and_then(|x| {
                        x.parse().map_err(|_| error!("Can't parse IP address"))
                        .ok()
                    });
                    peer.primary_addr = addr;
                    addr.map(|addr| {
                        // TODO(tailhook)
                        // self.remote.touch(id);
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
                self.send_gossip(addr);
            });
        }
    }
    fn ping_hosts(&mut self) {
        let tm = time_ms();
        if self.ping_queue.len() == 0 {
            let info = self.info.lock().expect("gossip not poisoned");
            if info.peers.len() == 0 {
                return // nothing to do
            }
            self.ping_queue = info.peers.keys().cloned().collect();
            thread_rng().shuffle(&mut self.ping_queue[..]);
        }
        for _ in 0..self.config.num_pings_to_send {
            if self.ping_queue.len() == 0 {
                break;
            }
            let id = self.ping_queue.pop().unwrap();
            // if not expired yet
            let addr = {
                let mut info = self.info.lock().expect("gossip not poisoned");
                info.peers.get_mut(&id).and_then(|peer| {
                    let peer = Arc::make_mut(peer);
                    if !peer.has_fresh_report(&self.config) {
                        let mut addr = peer.primary_addr;
                        if addr.is_none() ||
                           !peer.ping_primary_address(&self.config)
                        {
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
                })
            };
            addr.map(|addr| {
                self.send_gossip(addr);
            });
        }
    }

    fn store_peers(&mut self) {
        let data = {
            let info = self.info.lock().expect("gossip not poisoned");
            let mut buf = Vec::with_capacity(1024);
            let addrs = info.peers.iter()
                .flat_map(|(_, peer)| peer.addresses.iter())
                .map(|x| Json::String(x.to_string()))
                .collect();
            write!(&mut buf, "{}", Json::Object(vec![
                (String::from("ip_addresses"), Json::Array(addrs)),
                ].into_iter().collect())).unwrap();
            buf.into_boxed_slice()
        }; // unlocks self.info to avoid potential deadlocks

        self.storage.store_peers(data);
    }
    fn retry_new_hosts(&mut self) {
        use self::AddrStatus::*;
        loop {
            match self.add_queue.peek() {
                Some(x) if x.deadline < Instant::now() => {}
                _ => break,
            }
            let mut host = self.add_queue.pop().unwrap();
            if let Some(&Available) = self.addr_status.get(&host.address) {
                continue;
            }
            if host.attempts > self.config.add_host_retry_times {
                error!("Host {} is unresponsive", host.address);
                continue;
            }
            host.timeout = self.config.add_host_next_sleep(host.timeout);
            host.deadline = Instant::now() + host.timeout;
            host.attempts += 1;
            self.addr_status.insert(host.address, PingSent);
            self.send_gossip(host.address);
            self.add_queue.push(host);
        }
    }
    fn remove_failed_nodes(&mut self) {
        let mut info = self.info.lock().expect("gossip not poisoned");
        info.peers = mem::replace(&mut info.peers, HashMap::new()).into_iter()
            .filter(|&(ref id, ref peer)| {
                if peer.should_remove(&self.config) {
                    warn!("Peer {} / {:?} is removed",
                        id.to_hex(), peer.addresses);
                    false
                } else {
                    true
                }
            }).collect();
    }
    fn garbage_collector(&mut self) {
        self.remove_failed_nodes();
        self.store_peers();
        self.update_metrics();
    }
    fn update_metrics(&mut self) {
        let info = self.info.lock().expect("gossip not poisoned");
        NUM_PEERS.set(info.peers.len() as i64);
        NUM_STALE.set(info.peers.values()
            .filter(|p| p.is_stale(&self.config)).count() as i64);
    }
}

impl Ord for FutureHost {
    fn cmp(&self, other: &FutureHost) -> Ordering {
        self.deadline.cmp(&other.deadline)
    }
}

impl PartialOrd for FutureHost {
    fn partial_cmp(&self, other: &FutureHost) -> Option<Ordering> {
        self.deadline.partial_cmp(&other.deadline)
    }
}

impl PartialEq for FutureHost {
    fn eq(&self, other: &FutureHost) -> bool {
        self.deadline.eq(&other.deadline)
    }
}
