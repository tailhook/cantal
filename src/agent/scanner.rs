use std::sync::{Arc, RwLock};
use std::io::timer::sleep;
use std::time::duration::Duration;

use super::stats::Stats;
use super::scan::machine;
use super::scan::processes;
use super::scan::time_ms;


pub fn scan_loop(stats: Arc<RwLock<Stats>>) {
    let mut process_cache = processes::ReadCache::new();
    loop {
        let start = time_ms();
        let value = machine::read();
        stats.write().unwrap().machine = value;
        let processes = processes::read(&mut process_cache);
        let scan_time = time_ms() - start;
        stats.write().unwrap().scan_time = scan_time;
        debug!("Scan time {} ms", scan_time);
        sleep(Duration::seconds(2));
    }
}
