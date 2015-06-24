use time::{Timespec};
use rustc_serialize::json::{Json, ToJson};




#[derive(Debug, Clone, RustcEncodable, RustcDecodable)]
pub struct Report {
    pub peers: u32,
}


#[derive(Debug, Clone, RustcEncodable, RustcDecodable, Default)]
pub struct Peer {
    pub host: Option<String>,
    pub addr: String,
    pub last_probe: Option<Timespec>,
    pub last_report: Option<Timespec>,
    pub last_roundtrip: Option<u64>,
    pub report: Option<Report>,
}


impl ToJson for Peer {
    fn to_json(&self) -> Json {
        Json::Object(vec![
            ("ip", self.addr.to_json()),
            ("hostname", self.host.to_json()),
            ("probe", self.last_probe.map(|x| x.sec).to_json()),
            ("report", self.last_report.map(|x| x.sec).to_json()),
            ("roundtrip", self.last_roundtrip.to_json()),
        ].into_iter().map(|(x, y)| (String::from(x), y)).collect())
    }
}
