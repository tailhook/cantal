use std::net::SocketAddr;
use rustc_serialize::json::{Json, ToJson};


#[derive(Debug, Clone, RustcEncodable, RustcDecodable)]
pub struct Report {
    pub peers: u32,
    pub has_remote: bool,
    // TODO(tailhook) add last scan time
}


#[derive(Debug, Clone)]
pub struct Peer {
    pub addr: SocketAddr,
    pub host: Option<String>,
    pub name: Option<String>,
    pub last_probe: Option<u64>,
    pub last_roundtrip: Option<(u64, u64)>,
    pub random_peer_roundtrip: Option<(SocketAddr, u64, u64)>,
    pub report: Option<(u64, Report)>,
    pub last_report_direct: Option<u64>,
}

impl Peer {
    pub fn new(addr: SocketAddr) -> Peer {
        return Peer {
            addr: addr,
            host: None,
            name: None,
            last_probe: None,
            report: None,
            last_report_direct: None,
            last_roundtrip: None,
            random_peer_roundtrip: None,
        }
    }
}

impl ToJson for Peer {
    fn to_json(&self) -> Json {
        Json::Object(vec![
            ("addr", format!("{}", self.addr).to_json()),
            ("hostname", self.host.to_json()),
            ("name", self.name.to_json()),
            ("report", self.report.as_ref()
                .map(|&(x, _)| x).to_json()),
            ("peers", self.report.as_ref()
                .map(|&(_, ref x)| x.peers).to_json()),
            ("has_remote", self.report.as_ref()
                .map(|&(_, ref x)| x.has_remote).to_json()),
            ("last_report_direct", self.last_report_direct.to_json()),

            ("probe", self.last_probe.map(|x| x).to_json()),
            ("roundtrip", self.last_roundtrip.map(|(_, v)| v).to_json()),
            ("random_peer_roundtrip", self.random_peer_roundtrip
                .map(|(addr, timestamp, rtt)| vec![
                    addr.to_string().to_json(),
                    timestamp.to_json(),
                    rtt.to_json(),
                    ]).to_json()),
        ].into_iter().map(|(x, y)| (String::from(x), y)).collect())
    }
}
