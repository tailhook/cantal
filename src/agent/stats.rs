use std::default::Default;

use libc::pid_t;

use super::scan::time_ms;
use super::scan;
use history::History;
use super::storage::StorageStats;


pub struct Stats {

    pub startup_time: u64,
    pub last_scan: u64,
    pub scan_duration: u32,
    pub boot_time: Option<u64>,
    pub pid: pid_t,

    // TODO(tailhook) move to separate dependencies items
    pub storage: StorageStats,
    pub history: History,
    pub processes: Vec<scan::processes::MinimalProcess>,
}

impl Stats {
    pub fn new() -> Stats {
        return Stats {
            startup_time: time_ms(),
            last_scan: 0,
            scan_duration: 0,
            pid: 0,
            boot_time: None,
            storage: Default::default(),
            history: History::new(),
            processes: Default::default(),
        };
    }
}

