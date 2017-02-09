use std::cmp::min;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use rand::{thread_rng, Rng};

use {HostId};
use gossip::Config;
use time_util::duration_to_millis;


pub struct ConfigBuilder {
    machine_id: Option<HostId>,
    cluster_name: Option<String>,
    name: Option<String>,
    hostname: Option<String>,
    bind: Option<SocketAddr>,
    addresses: Vec<SocketAddr>,

    /// Wake up once per 1000 ms to send few probes
    interval: Duration,
    /// Number of probes to send at each interval
    num_pings_to_send: u64,

    /// If we got any probe or report during 5 seconds, don't probe this node
    min_ping_interval: Duration,

    /// But if we sent no probe within 60 seconds (but were receiving reports,
    /// so didn't hit 5 seconds timeout above), we should send probe anyway.
    /// This allows too keep roundtrip times on both nodes reasonably up to
    /// date
    max_ping_interval: Duration,

    /// Num of friend nodes to send within each request, everything must fit
    /// MAX_PACKET_SIZE which is capped at maximum UDP packet size (65535),
    /// better if it fits single IP packet (< 1500)
    num_friends_in_a_packet: usize,

    /// After we had no reports from node for 20 seconds (but we sent probe
    /// during this time) we consider node to be inaccessible by it's primary
    /// IP and are trying to reach it by pinging any other random IP address.
    prefail_time: u64,


    /// Maximum expected roundtrip time. We consider report failing if it's not
    /// received during this time. Note, this doesn't need to be absolute
    /// ceiling of RTT, and we don't do any crazy things based on the timeout,
    /// this is just heuristic for pre-fail condition.
    max_roundtrip: u64,

    /// After this time we consider node failing and don't send it in
    /// friendlist.  Note that all nodes that where up until we marked node as
    /// failinig do know the node, and do ping it. This is currently used only
    fail_time: u64,


    /// This is time after last heartbeat when node will be removed from the
    /// list of known nodes. This should be long after FAIL_TIME. (But not
    /// necessarily 48x longer, as we do now).  Also note that node will be
    /// removed from all peers after FAIL_TIME + REMOVE_TIME +
    /// longest-round-trip-time
    remove_time: u64,

    /// This is a size of our UDP buffers. The maximum value depends on
    /// NUM_FRIENDS and the number of IP addresses at each node. It's always
    /// capped at maximum UDP packet size of 65535
    max_packet_size: usize,

    /// Number of times to retry failed AddHost
    add_host_retry_times: u32,
    /// A timeout of the first retry
    add_host_retry_min: Duration,
    /// The exponent of the next retry
    add_host_retry_exponent: f32,
    /// Maximum retry timeout (after randomization)
    add_host_retry_cap: Duration,
    /// Randomization coefficients, e.g. (0.5, 1.5)
    ///
    /// This is used so that lots of pings were not sent at the same time
    add_host_retry_random: (f32, f32),
}

impl Config {
    pub fn new() -> ConfigBuilder {
        ConfigBuilder {
            machine_id: None,
            cluster_name: None,
            name: None,
            hostname: None,
            bind: None,
            addresses: Vec::new(),

            interval: Duration::new(1, 0),
            num_pings_to_send: 10,
            min_ping_interval: Duration::new(5, 0),
            max_ping_interval: Duration::new(60, 0),
            num_friends_in_a_packet: 10,
            prefail_time: 20_000,
            max_roundtrip: 2000,
            fail_time: 3600_000,
            remove_time: 2 * 86400_000,
            max_packet_size: 8192,

            add_host_retry_times: 5,
            add_host_retry_min: Duration::from_millis(100),
            add_host_retry_exponent: 2.0,
            add_host_retry_cap: Duration::new(3600, 0),
            add_host_retry_random: (0.5, 1.5),
        }
    }

    pub fn add_host_first_sleep(&self) -> Duration {
        let ms = duration_to_millis(self.add_host_retry_min) as f32;
        let (low, high) = self.add_host_retry_random;
        let rnd_ms = thread_rng().gen_range(ms*low, ms*high) as u64;
        return Duration::from_millis(rnd_ms);
    }

    pub fn add_host_next_sleep(&self, prev_sleep: Duration) -> Duration {
        let ms = (duration_to_millis(prev_sleep) as f32)
            * self.add_host_retry_exponent;
        let (low, high) = self.add_host_retry_random;
        let rnd_ms = thread_rng().gen_range(ms*low, ms*high) as u64;
        return min(Duration::from_millis(rnd_ms), self.add_host_retry_cap);
    }

}

impl ConfigBuilder {
    pub fn bind(&mut self, addr: SocketAddr) -> &mut Self {
        self.bind = Some(addr);
        self
    }
    pub fn cluster_name(&mut self, name: &str) -> &mut Self {
        self.cluster_name = Some(name.into());
        self
    }
    pub fn name(&mut self, name: &str) -> &mut Self {
        self.name = Some(name.into());
        self
    }
    pub fn hostname(&mut self, name: &str) -> &mut Self {
        self.hostname = Some(name.into());
        self
    }
    pub fn machine_id(&mut self, machine_id: &HostId) -> &mut Self {
        self.machine_id = Some(machine_id.clone());
        self
    }
    pub fn addresses(&mut self, addresses: &[SocketAddr]) -> &mut Self {
        self.addresses = addresses.to_vec();
        self
    }
    pub fn done(&mut self) -> Arc<Config> {
        Arc::new(Config {
            machine_id: self.machine_id.clone().expect("machine_id"),
            cluster_name: Arc::new(
                self.cluster_name.clone().expect("cluster_name")),
            hostname: Arc::new(self.hostname.clone().expect("hostname")),
            name: Arc::new(self.name.clone().expect("name")),
            bind: self.bind.expect("bind address"),
            addresses: self.addresses.clone(),
            str_addresses: Arc::new(
                self.addresses.iter().map(ToString::to_string).collect()),

            interval: self.interval,
            num_pings_to_send: self.num_pings_to_send,
            min_ping_interval: self.min_ping_interval,
            max_ping_interval: self.max_ping_interval,
            num_friends_in_a_packet: self.num_friends_in_a_packet,
            prefail_time: self.prefail_time,
            max_roundtrip: self.max_roundtrip,
            fail_time: self.fail_time,
            remove_time: self.remove_time,
            max_packet_size: self.max_packet_size,

            add_host_retry_times: self.add_host_retry_times,
            add_host_retry_min: self.add_host_retry_min,
            add_host_retry_exponent: self.add_host_retry_exponent,
            add_host_retry_cap: self.add_host_retry_cap,
            add_host_retry_random: self.add_host_retry_random,
        })
    }
}
