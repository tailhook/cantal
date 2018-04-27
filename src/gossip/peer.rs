use std::collections::HashSet;
use std::net::SocketAddr;
use std::sync::Arc;

use rand::{thread_rng};
use rand::seq::sample_iter;

use super::super::scan::time_ms;
use gossip::Config;
use time_util::duration_to_millis;
use id::Id;


// TODO(tailhook) probably remove the structure
#[derive(Debug, Clone, Deserialize, Serialize, RustcEncodable, RustcDecodable)]
pub struct Report {
    pub peers: u32,
    pub has_remote: bool,
    // TODO(tailhook) add last scan time
}


#[derive(Serialize, Debug, Clone)]
pub struct Peer {
    pub id: Id,
    pub known_since: u64,
    /// Primary IP is address used to send gossip packets to
    /// It's derived from the address this machine has sent packets from
    pub primary_addr: Option<SocketAddr>,
    /// All addresses declared by host, including virtual ones
    pub addresses: HashSet<SocketAddr>,
    pub hostname: Option<String>,
    pub name: Option<String>,
    pub last_probe: Option<(u64, SocketAddr)>,
    pub probes_sent: u64,
    pub pings_received: u64,
    pub pongs_received: u64,
    pub last_roundtrip: Option<(SocketAddr, u64, u64)>,
    pub random_peer_roundtrip: Option<(SocketAddr, u64, u64)>,
    pub report: Option<(u64, Report)>,
    pub last_report_direct: Option<u64>,
}

impl Peer {
    pub fn new(id: Id) -> Peer {
        Peer {
            id: id,
            known_since: time_ms(),
            primary_addr: None,
            addresses: HashSet::new(),
            hostname: None,
            name: None,
            last_probe: None,
            probes_sent: 0,
            pings_received: 0,
            pongs_received: 0,
            last_roundtrip: None,
            random_peer_roundtrip: None,
            report: None,
            last_report_direct: None,
        }
    }

    pub fn apply_addresses<I:Iterator<Item=SocketAddr>>(&mut self,
            addrs: I, direct: bool)
    {
        if direct {
            self.addresses = addrs.collect();
        } else {
            self.addresses.extend(addrs);
        }
    }

    pub fn apply_report(&mut self, src: Option<(u64, Report)>, direct: bool) {
        let overwrite = match (&self.report, &src) {
            // apply only newer report
            (&Some((ots, _)), &Some((nts, _))) if ots < nts => true,
            // or if one did not exists
            (&None, &Some((_, _))) => true,
            _ => false,
        };
        if overwrite {
            if direct {
                self.last_report_direct = src.as_ref().map(|&(x, _)| x);
            }
            self.report = src;
        }
    }
    pub fn apply_hostname(&mut self, hostname: Option<&str>,
        direct: bool)
    {
        let overwrite = match (&self.hostname, &hostname) {
            (&None, &Some(_)) => true,
            (&Some(ref x), &Some(ref y)) if x != y => {
                warn!("Host {} has hostname {:?} but received {:?} for it. {}",
                    self.id, x, y,
                    if direct { "Overwriting..." } else { "Ignoring..." });
                direct
            }
            _ => false,
        };
        if overwrite {
            self.hostname = hostname.map(|x| x.to_string());
        }
    }
    pub fn apply_node_name(&mut self, name: Option<&str>, direct: bool)
    {
        let overwrite = match (&self.name, &name) {
            (&None, &Some(_)) => true,
            (&Some(ref x), &Some(ref y)) if x != y => {
                warn!("Host {} has node name {:?} but received {:?} for it. {}",
                    self.id, x, y,
                    if direct { "Overwriting..." } else { "Ignoring..." });
                direct
            }
            _ => false,
        };
        if overwrite {
            self.name = name.map(|x| x.to_string());
        }
    }
    pub fn apply_roundtrip(&mut self, rtt: (u64, u64),
        source: SocketAddr, direct: bool)
    {
        if direct {
            self.last_roundtrip = Some((source, rtt.0, rtt.1));
        } else {
            match self.random_peer_roundtrip {
                Some((_, tm, _)) if tm < rtt.0 => {
                    self.random_peer_roundtrip = Some((source, rtt.0, rtt.1));
                }
                Some(_) => {}
                None => {
                    self.random_peer_roundtrip = Some((source, rtt.0, rtt.1));
                }
            }
        }
    }

    pub fn has_fresh_report(&self, config: &Arc<Config>) -> bool {
        let now = time_ms();
        let min_probe = duration_to_millis(config.min_ping_interval);
        let max_probe = duration_to_millis(config.max_ping_interval);
        match self.report {
            // never reported
            None => { return false; }
            // outdated report
            Some((ts, _)) if ts + min_probe < now => { return false; }
            _ => {}
        }
        // In case we have fresh report (probably pushed from host or from
        // third party peer), we need to have reasonably fresh roundtrip time
        match self.last_probe {
            // never reported
            None => { return false; }
            // outdated
            Some((ts, _)) if ts + max_probe < now => { return false; }
            _ => {}
        }
        return true;
    }

    pub fn ping_primary_address(&self, config: &Arc<Config>) -> bool {
        let now = time_ms();
        let max_roundtrip = config.max_roundtrip;
        let prefail_time = config.prefail_time;
        match self.last_probe {
            // never probed (yet)
            None => { return true; }
            // not yet responed
            Some((ts, _)) if ts + max_roundtrip > now => { return true; }
            _ => {}
        }
        match self.last_report_direct {
            // no report received ever
            None => false,
            // last report is recently received
            Some(ts) if ts + prefail_time > now => true,
            _ => false,
        }
    }

    pub fn random_ping_addr(&self) -> Option<SocketAddr> {
        if let Some((_, ref addr)) = self.last_probe {
            // exclude last probe address to facilitate quick scanning
            let mut list = sample_iter(&mut thread_rng(),
                self.addresses.iter().filter(|x| *x != addr).cloned(), 1);
            Some(list.ok().and_then(|mut x| x.pop()).unwrap_or(*addr))
        } else {
            let mut list = sample_iter(&mut thread_rng(),
                self.addresses.iter().cloned(), 1);
            list.ok().and_then(|mut list| list.pop())
        }
    }

    pub fn is_failing(&self, config: &Arc<Config>) -> bool {
        let now = time_ms();
        match self.report {
            // never probed (yet)
            None => self.known_since + config.fail_time < now,
            // not yet responed
            Some((ts, _)) => ts + config.fail_time < now,
        }
    }

    pub fn is_stale(&self, config: &Arc<Config>) -> bool {
        let now = time_ms();
        match self.report {
            // never probed (yet)
            None => self.known_since + config.stale_time < now,
            Some((ts, _)) => ts + config.stale_time < now,
        }
    }

    pub fn should_remove(&self, config: &Arc<Config>) -> bool {
        let now = time_ms();
        match self.report {
            // never probed (yet)
            None => self.known_since + config.remove_time < now,
            // not yet responed
            Some((ts, _)) => ts + config.remove_time < now,
        }
    }
}
