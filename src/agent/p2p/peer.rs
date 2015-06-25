use std::net::SocketAddr;
use time::{Timespec};
use rustc_serialize::json::{Json, ToJson};




#[derive(Debug, Clone, RustcEncodable, RustcDecodable)]
pub struct Report {
    pub peers: u32,
    // TODO(tailhook) add last scan time
}


#[derive(Debug, Clone)]
pub struct Peer {
    pub addr: SocketAddr,
    pub host: Option<String>,
    pub last_probe: Option<Timespec>,
    pub last_report: Option<Timespec>,
    pub last_roundtrip: Option<(Timespec, u64)>,
    pub report: Option<Report>,
}

impl Peer {
    pub fn new(addr: SocketAddr) -> Peer {
        return Peer {
            addr: addr,
            host: None,
            last_probe: None,
            last_report: None,
            last_roundtrip: None,
            report: None,
        }
    }
}

impl ToJson for Peer {
    fn to_json(&self) -> Json {
        Json::Object(vec![
            ("addr", format!("{}", self.addr).to_json()),
            ("hostname", self.host.to_json()),
            ("peers", self.report.as_ref().map(|x| x.peers).to_json()),
            ("probe", self.last_probe.map(|x| x.sec).to_json()),
            ("report", self.last_report.map(|x| x.sec).to_json()),
            ("roundtrip", self.last_roundtrip.map(|(_, v)| v).to_json()),
        ].into_iter().map(|(x, y)| (String::from(x), y)).collect())
    }
}
