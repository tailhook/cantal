mod config;
mod proto;
mod errors;
mod peer;
mod info;
mod constants;

use std::net::SocketAddr;
use std::sync::Arc;

use tk_easyloop;
use quick_error::ResultExt;

use {HostId};

pub use self::errors::InitError;


/// Fields are documented in `config.rs`
pub struct Config {
    machine_id: HostId,
    cluster_name: Arc<String>,
    hostname: Arc<String>,
    name: Arc<String>,
    bind: SocketAddr,
    addresses: Vec<SocketAddr>,
    str_addresses: Arc<Vec<String>>,

    interval: u64,
    num_probes: u64,
    min_probe: u64,
    max_probe: u64,
    num_friends: usize,
    prefail_time: u64,
    max_roundtrip: u64,
    fail_time: u64,
    remove_time: u64,
    max_packet_size: usize,
}


pub fn spawn(cfg: &Arc<Config>) -> Result<(), InitError> {
    tk_easyloop::spawn(proto::Proto::bind(cfg)?);
    Ok(())
}

