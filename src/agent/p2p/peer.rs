use time::{Timespec, Duration};
use std::net::SocketAddr;


#[derive(Debug, Clone, RustcDecodable, RustcEncodable)]
pub struct Probe {
    roundtrip: u32,
    peers: u32,
}


#[derive(Debug, Clone, RustcDecodable, RustcEncodable)]
pub struct Peer {
    host: Option<String>,
    addr: String,
    last_probe: Option<Timespec>,
    last_report: Option<Timespec>,
    probe: Option<Probe>,
}

