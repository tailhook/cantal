use std::sync::{Arc, RwLock};
use std::old_io::timer::sleep;
use std::time::duration::Duration;

use super::stats::Stats;
use super::scan::Tip;
use super::scan::machine;
use super::scan::processes;
use super::scan::time_ms;


pub fn scan_loop(stats: Arc<RwLock<Stats>>) {
    let mut process_cache = processes::ReadCache::new();
    loop {
        let start = time_ms();
        let mut tip = Tip::new();

        let boot_time = machine::read(&mut tip);
        if let Ok(ref mut stats) = stats.write() {
            stats.boot_time = boot_time.or(stats.boot_time);
        }


        let processes = processes::read(&mut process_cache);
        stats.write().unwrap().processes = processes;

        stats.write().unwrap().tip = tip;
        let scan_time = time_ms() - start;
        stats.write().unwrap().scan_time = scan_time;
        debug!("Scan time {} ms", scan_time);
        sleep(Duration::seconds(2));
    }
}
