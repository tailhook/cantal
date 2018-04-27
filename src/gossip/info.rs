use std::sync::Arc;
use std::net::SocketAddr;
use std::collections::HashMap;

use rand::{thread_rng};
use rand::seq::sample_iter;

use {HostId};
use gossip::Config;
use gossip::peer::Peer;
use gossip::proto::FriendInfo;


pub struct Info {
    pub peers: HashMap<HostId, Arc<Peer>>,
    pub has_remote: bool,
}

impl Info {
    pub fn new() -> Info {
        Info {
            peers: HashMap::new(),
            has_remote: false,
        }
    }
    pub fn get_friends(&self, exclude: SocketAddr, config: &Arc<Config>)
        -> Vec<FriendInfo>
    {
        let mut rng = thread_rng();
        let other_peers = self.peers.values()
            .filter(|peer| !peer.addresses.contains(&exclude))
            .filter(|peer| !peer.is_failing(config));
        let friends = sample_iter(&mut rng, other_peers,
            config.num_friends_in_a_packet)
            .unwrap_or_else(|x| x);
        friends.into_iter().map(|f| FriendInfo {
            id: f.id.clone(),
            my_primary_addr: f.primary_addr.map(|x| format!("{}", x)),
            addresses: f.addresses.iter().map(|x| format!("{}", x)).collect(),
            host: f.host.clone(),
            name: f.name.clone(),
            report: f.report.clone(),
            roundtrip: f.last_roundtrip.map(|(_, ts, rtt)| (ts, rtt)),
        }).collect()
    }
}

