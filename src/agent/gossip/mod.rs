mod config;
mod proto;
mod errors;
mod peer;
mod info;
mod constants;  // TODO(tailhook) to remove
mod command;
mod public;

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use futures::stream::Stream;
use futures::sync::mpsc::{unbounded as channel, UnboundedReceiver};
use quick_error::ResultExt;
use tk_easyloop;
use void::Void;

use {HostId};

pub use self::errors::InitError;
pub use self::public::Gossip;


/// Fields are documented in `config.rs`
pub struct Config {
    machine_id: HostId,
    cluster_name: Arc<String>,
    hostname: Arc<String>,
    name: Arc<String>,
    bind: SocketAddr,
    addresses: Vec<SocketAddr>,
    str_addresses: Arc<Vec<String>>,

    interval: Duration,
    num_probes: u64,
    min_probe: u64,
    max_probe: u64,
    num_friends: usize,
    prefail_time: u64,
    max_roundtrip: u64,
    fail_time: u64,
    remove_time: u64,
    max_packet_size: usize,

    add_host_retry_times: u32,
    add_host_retry_min: Duration,
    add_host_retry_exponent: f32,
    add_host_retry_cap: u32,
    add_host_retry_random: (f32, f32),
}

pub struct GossipInit {
    receiver: UnboundedReceiver<command::Command>,
    config: Arc<Config>,
}

pub fn init(cfg: &Arc<Config>) -> (Gossip, GossipInit) {
    let (tx, rx) = channel();
    return (
        public::new(tx),
        GossipInit {
            receiver: rx,
            config: cfg.clone(),
        }
    );
}

impl GossipInit {
    pub fn spawn(self) -> Result<(), InitError> {
        let rx = self.receiver
            .map_err(|_| -> Void { panic!("gossip stream canceled") });
        tk_easyloop::spawn(proto::Proto::bind(&self.config, rx)?);
        Ok(())
    }
}
