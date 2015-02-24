use std::default::Default;
use super::scan::time_ms;
use super::scan;


#[derive(Encodable)]
pub struct Stats {
    pub startup_time: u64,
    pub scan_time: u64,
    pub machine: scan::machine::MachineStats,
    pub processes: scan::processes::Processes,
}

impl Stats {
    pub fn new() -> Stats {
        return Stats {
            startup_time: time_ms(),
            scan_time: 0,
            machine: Default::default(),
            processes: Default::default(),
        };
    }
}
