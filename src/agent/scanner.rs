use std::sync::{Arc, RwLock};
use std::old_io::timer::sleep;
use std::time::duration::Duration;

use super::stats::Stats;
use super::scan::Tip;
use super::scan::machine;
use super::scan::processes;
use super::scan::values;
use super::scan::time_ms;


pub fn scan_loop(stats: Arc<RwLock<Stats>>) {
    let mut process_cache = processes::ReadCache::new();
    let mut values_cache = values::ReadCache::new();
    loop {
        let start = time_ms();
        let mut tip = Tip::new();

        let boot_time = machine::read(&mut tip);

        let processes = processes::read(&mut process_cache);
        values::read(&mut tip, &mut values_cache, &processes);

        let scan_time = time_ms() - start;
        stats.write().unwrap().scan_time = scan_time;
        debug!("Got {} values and {} processes in {} ms",
            tip.len(), processes.len(), scan_time);

        if let Ok(ref mut stats) = stats.write() {
            stats.boot_time = boot_time.or(stats.boot_time);
            stats.tip = tip;
            stats.processes = processes;
        }

        sleep(Duration::seconds(2));
    }
}
