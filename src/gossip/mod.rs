mod config;
mod proto;
mod errors;
mod peer;
mod info;
mod command;
mod public;

use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use futures::Future;
use futures::stream::Stream;
use futures::sync::mpsc::{unbounded as channel, UnboundedReceiver};
use tk_easyloop;
use void::Void;

use id::Id;
use incoming::Incoming;
use remote::{self, Hostname};
use storage::Storage;

pub use self::peer::Peer;
pub use self::errors::InitError;
pub use self::public::{Gossip, noop};
pub use self::info::Info;
pub use self::proto::{NUM_PEERS, NUM_STALE};


/// Fields are documented in `config.rs`
pub struct Config {
    machine_id: Id,
    cluster_name: Arc<String>,
    hostname: Hostname,
    name: Arc<String>,
    bind: SocketAddr,
    #[allow(dead_code)]
    addresses: Vec<SocketAddr>,
    str_addresses: Arc<Vec<String>>,

    interval: Duration,
    num_pings_to_send: u64,
    min_ping_interval: Duration,
    max_ping_interval: Duration,
    num_friends_in_a_packet: usize,
    prefail_time: u64,
    max_roundtrip: u64,
    fail_time: u64,
    stale_time: u64,
    remove_time: u64,
    max_packet_size: usize,

    garbage_collector_interval: Duration,

    add_host_retry_times: u32,
    add_host_retry_min: Duration,
    add_host_retry_exponent: f32,
    add_host_retry_cap: Duration,
    add_host_retry_random: (f32, f32),
}

pub struct GossipInit {
    receiver: UnboundedReceiver<command::Command>,
    info: Arc<Mutex<Info>>,
    config: Arc<Config>,
}

pub fn init(cfg: &Arc<Config>) -> (Gossip, GossipInit) {
    let (tx, rx) = channel();
    let info = Arc::new(Mutex::new(Info::new()));
    return (
        public::new(tx, &info),
        GossipInit {
            receiver: rx,
            info: info,
            config: cfg.clone(),
        }
    );
}

impl GossipInit {
    pub fn spawn(self, storage: &Arc<Storage>,
        incoming: &Incoming, remote: &remote::Remote)
        -> Result<(), InitError>
    {
        let rx = self.receiver
            .map_err(|_| -> Void { panic!("gossip stream canceled") });
        tk_easyloop::spawn(proto::Proto::new(
            &self.info,
            &self.config,
            rx,
            storage,
            incoming,
            remote,
        )?.then(|res| -> Result<(), ()> {
            error!("FATAL ERROR: Gossip future is exited: {:?}", res);
            panic!("gossip future is exited");
        }));
        Ok(())
    }
}
