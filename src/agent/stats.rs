use std::default::Default;
use super::scan::time_ms;
use super::scan;


#[derive(Encodable)]
pub struct Stats {
    pub startup_time: u64,
    pub machine: scan::machine::MachineStats,
}

impl Stats {
    pub fn new() -> Stats {
        return Stats {
            startup_time: time_ms(),
            machine: Default::default(),
        };
    }
}
