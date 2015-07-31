use std::iter::repeat;
use std::ops::Deref;
use std::net::SocketAddr;

use libc::pid_t;
use unicase::UniCase;
use byteorder::{BigEndian, ByteOrder};
use hyper::header::{Upgrade, ProtocolName};
use hyper::header::{Connection};
use hyper::version::HttpVersion as Version;
use hyper::header::ConnectionOption::ConnectionHeader;
use websocket::header::{WebSocketVersion, WebSocketKey};
use rustc_serialize::json;

use super::http;
use super::scan::time_ms;
use super::remote::Peers;
use super::p2p::GossipStats;
use super::http::{Request, BadRequest};
use super::util::Consume;
use super::server::{Context};
use super::stats::Stats;
use super::deps::{Dependencies, LockedDeps};
use super::rules;


#[derive(RustcEncodable, RustcDecodable, Debug, Clone)]
pub struct Beacon {
    pub pid: pid_t,
    pub current_time: u64,
    pub startup_time: u64,
    pub boot_time: Option<u64>,
    pub scan_time: u64,
    pub scan_duration: u32,
    pub processes: usize,
    pub values: usize,
    pub peers: usize,
    pub fine_history_length: usize,
    pub history_age: u64,
    pub remote_total: Option<usize>,
    pub remote_connected: Option<usize>,
    pub peers_with_remote: usize,
}

#[derive(RustcEncodable, RustcDecodable, Debug)]
pub enum OutputMessage {
    Beacon(Beacon),
    NewPeer(String),
    Stats(rules::RawResult),
}

#[derive(RustcEncodable, RustcDecodable, Debug)]
pub enum InputMessage {
    Subscribe(rules::RawRule, usize),
    Unsubscribe(rules::RawRule),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Opcode {
    Text,
    Binary,
}

impl Opcode {
    pub fn from(src: u8) -> Option<Opcode> {
        match src {
            1 => Some(Opcode::Text),
            2 => Some(Opcode::Binary),
            _ => None,
        }
    }
}


pub fn respond_websock(req: &Request, _context: &mut Context)
    -> Result<http::Response, Box<http::Error>>
{
    if req.version != Version::Http11 {
        return Err(BadRequest::err("Unsupported request HTTP version"));
    }

    if req.headers.get() != Some(&(WebSocketVersion::WebSocket13)) {
        return Err(BadRequest::err("Unsupported WebSocket version"));
    }

    let key  = match req.headers.get::<WebSocketKey>() {
        Some(key) => key,
        None => {
            return Err(BadRequest::err("Missing Sec-WebSocket-Key"));
        }
    };

    match req.headers.get() {
        Some(&Upgrade(ref upgrade)) => {
            let mut correct_upgrade = false;
            for u in upgrade {
                if u.name == ProtocolName::WebSocket {
                    correct_upgrade = true;
                }
            }
            if !correct_upgrade {
                return Err(BadRequest::err(
                    "Invalid Upgrade WebSocket header"));
            }
        }
        None => {
            return Err(BadRequest::err("Missing Upgrade header"));
        }
    };

    match req.headers.get() {
        Some(&Connection(ref connection)) => {
            if !connection.contains(&(ConnectionHeader(
                UniCase("Upgrade".to_string()))))
            {
                return Err(BadRequest::err(
                    "Invalid Connection WebSocket header"));
            }
        }
        None => {
            return Err(BadRequest::err(
                "Missing Connection WebSocket header"));
        }
    }

    Ok(http::Response::accept_websock(key))
}

pub fn parse_message<F, T>(buf: &mut Vec<u8>, context: &mut Context, cb: F)
    -> Option<T>
    where F: FnOnce(Opcode, &[u8], &mut Context) -> Option<T>
{
    if buf.len() < 2 {
        return None;
    }
    let fin = buf[0] & 0b10000000 != 0;
    let opcode = buf[0] & 0b00001111;
    let mask = buf[1] & 0b10000000 != 0;
    let mut ln = (buf[1] & 0b01111111) as usize;
    let mut pref = 2;
    let mut result = None;
    if ln == 126 {
        if buf.len() < 4 {
            return None;
        }
        ln = BigEndian::read_u16(&buf[2..4]) as usize;
        pref = 4;
    } else if ln == 127 {
        if buf.len() < 10 {
            return None;
        }
        ln = BigEndian::read_u64(&buf[2..10]) as usize;
        pref = 10;
    }
    if buf.len() < pref + ln + (if mask { 4 } else { 0 }) {
        return None;
    }
    if mask {
        let mask = buf[pref..pref+4].to_vec(); // TODO(tailhook) optimize
        pref += 4;
        for (m, t) in mask.iter().cycle().zip(buf[pref..pref+ln].iter_mut()) {
            *t ^= *m;
        }
    }
    {
        if !fin {
            warn!("Partial frames are not supported");
        } else {
            result = match Opcode::from(opcode) {
                None => {
                    warn!("Invalid opcode {:?}", opcode);
                    None
                }
                Some(op) => cb(op, &buf[pref..pref+ln], context),
            }
        }
    }
    buf.consume(pref + ln);
    result
}

pub fn write_text(buf: &mut Vec<u8>, chunk: &str) {
    // TODO(tailhook) implement masking for client websock
    // as it should be required (by spec)
    let bytes = chunk.as_bytes();
    buf.push(0b10000001);  // text message
    if bytes.len() > 65535 {
        buf.push(127);
        let start = buf.len();
        buf.extend(repeat(0).take(8));
        BigEndian::write_u64(&mut buf[start ..],
                             bytes.len() as u64);
    } else if bytes.len() > 125 {
        buf.push(126);
        let start = buf.len();
        buf.extend(repeat(0).take(2));
        BigEndian::write_u16(&mut buf[start ..],
                             bytes.len() as u16);
    } else {
        buf.push(bytes.len() as u8);
    }
    buf.extend(bytes.iter().cloned());
}

pub fn beacon(deps: &Dependencies) -> String {
    // Lock one by one, to avoid deadlocks
    let (pid,
         startup_time,
         boot_time,
         scan_time,
         scan_duration,
         processes,
         values,
         fine_history_length,
         history_age) = {
            let st = deps.read::<Stats>();
            (   st.pid,
                st.startup_time,
                st.boot_time.map(|x| x*1000),
                st.last_scan,
                st.scan_duration,
                st.processes.len(),
                st.history.tip.len() + st.history.fine.len() +
                       st.history.coarse.len(),
                st.history.fine_timestamps.len(),
                st.history.age)
    };
    let (gossip_peers, peers_with_remote) = {
        let gossip = deps.read::<GossipStats>();
        (gossip.peers.len(), gossip.num_having_remote)
    };
    let (remote_total, remote_connected) =
        if let &Some(ref peers) = deps.read::<Option<Peers>>().deref() {
            (Some(peers.addresses.len()), Some(peers.connected))
        } else {
            (None, None)
        };
    json::encode(&OutputMessage::Beacon(Beacon {
        pid: pid,
        current_time: time_ms(),
        startup_time: startup_time,
        boot_time: boot_time,
        scan_time: scan_time,
        scan_duration: scan_duration,
        processes: processes,
        values: values,
        fine_history_length: fine_history_length,
        history_age: history_age,
        peers: gossip_peers,
        remote_total: remote_total,
        remote_connected: remote_connected,
        peers_with_remote: peers_with_remote,
    })).unwrap()
}

pub fn new_peer(peer: SocketAddr) -> String {
    json::encode(&OutputMessage::NewPeer(format!("{}", peer))).unwrap()
}
