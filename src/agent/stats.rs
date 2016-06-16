use std::default::Default;

use libc::pid_t;

use super::scan::time_ms;
use super::scan;
use history::History;
use super::storage::StorageStats;


pub struct Stats {
    pub pid: pid_t,
    pub id_hex: String,
    pub addresses_str: Vec<String>,
    pub name: String,
    pub hostname: String,
    pub cluster_name: Option<String>,

    pub startup_time: u64,
    pub last_scan: u64,
    pub scan_duration: u32,
    pub boot_time: Option<u64>,

    pub storage: StorageStats,
    pub history: History,
    pub processes: Vec<scan::processes::MinimalProcess>,
    pub connections: Option<scan::connections::Connections>,
}

impl Stats {
    pub fn new(pid: pid_t, name: String, hostname: String,
        cluster_name: Option<String>, id_hex: String,
        addresses_str: Vec<String>)
        -> Stats
    {
        return Stats {
            pid: pid,
            id_hex: id_hex,
            addresses_str: addresses_str,
            name: name,
            hostname: hostname,
            cluster_name: cluster_name,
            startup_time: time_ms(),
            last_scan: 0,
            scan_duration: 0,
            boot_time: None,
            storage: Default::default(),
            history: History::new(),
            processes: Default::default(),
            connections: Default::default(),
        };
    }
}

