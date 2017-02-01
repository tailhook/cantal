use std::sync::{Arc, Mutex};
use std::net::SocketAddr;

use cbor::Encoder;
use futures::{Future, Async};
use tk_easyloop;
use quick_error::ResultExt;
use tokio_core::net::UdpSocket;

use {HostId};
use time_util::time_ms;
use gossip::Config;
use gossip::errors::InitError;
use gossip::peer::{Report};
use gossip::info::Info;
use gossip::constants::MAX_PACKET_SIZE;


pub struct Proto {
    sock: UdpSocket,
    config: Arc<Config>,
    info: Arc<Mutex<Info>>,
}

#[derive(Debug, Clone, RustcEncodable, RustcDecodable)]
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

#[derive(Debug, Clone, RustcEncodable, RustcDecodable)]
pub struct MyInfo {
    id: HostId,
    addresses: Arc<Vec<String>>,
    host: Arc<String>,
    name: Arc<String>,
    report: Report,
}

#[derive(Debug, Clone, RustcEncodable, RustcDecodable)]
pub struct FriendInfo {
    pub id: HostId,
    pub my_primary_addr: Option<String>,
    pub addresses: Vec<String>,
    pub host: Option<String>,
    pub name: Option<String>,
    pub report: Option<(u64, Report)>,
    pub roundtrip: Option<(u64, u64)>,
}


impl Proto {
    pub fn bind(config: &Arc<Config>)
       -> Result<Proto, InitError>
    {
        let s = UdpSocket::bind(&config.bind, &tk_easyloop::handle())
            .context(config.bind)?;
        let info = Info::new();
        Ok(Proto {
            sock: s,
            config: config.clone(),
            info: Arc::new(Mutex::new(info)),
        })
    }
}

impl Future for Proto {
    type Item = ();
    type Error = ();
    fn poll(&mut self) -> Result<Async<()>, ()> {

        Ok(Async::NotReady)
    }
}

impl Proto {
    fn send_gossip(&mut self, addr: SocketAddr) {
        debug!("Sending gossip {}", addr);
        let mut buf = Vec::with_capacity(MAX_PACKET_SIZE);
        {
            let info = self.info.lock().expect("gossip info poisoned");
            let mut e = Encoder::from_writer(&mut buf);
            e.encode(&[&Packet::Ping {
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
                friends: info.get_friends(addr),
            }]).unwrap();
        }
        if buf.len() >= MAX_PACKET_SIZE {
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
}
