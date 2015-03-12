use std::sync::{RwLock};
use std::fs::File;
use std::io::Write;
use std::old_io::timer::sleep;
use std::time::duration::Duration;

use msgpack::Encoder as Mencoder;

use super::stats::Stats;
use super::scan::Tip;
use super::scan::machine;
use super::scan::processes;
use super::scan::values;
use super::scan::time_ms;
use super::util::Cell;
use super::storage::Buffer;


const SNAPSHOT_INTERVAL: u64 = 60000;


pub fn scan_loop(stats: &RwLock<Stats>, cell: Option<&Cell<Buffer>>) {
    let mut last_store = time_ms();
    let mut process_cache = processes::ReadCache::new();
    let mut values_cache = values::ReadCache::new();
    loop {
        let start = time_ms();
        let mut tip = Tip::new();

        let boot_time = machine::read(&mut tip);

        let processes = processes::read(&mut process_cache);
        values::read(&mut tip, &mut values_cache, &processes);

        let scan_duration = (time_ms() - start) as u32;

        if let Ok(ref mut stats) = stats.write() {
            stats.scan_duration = scan_duration;
            debug!("Got {} values and {} processes in {} ms",
                tip.map.len(), processes.len(), scan_duration);

            stats.history.push(start, scan_duration, tip);
            stats.boot_time = boot_time.or(stats.boot_time);
            stats.processes = processes;

            if start - last_store > SNAPSHOT_INTERVAL {
                last_store = start;
                if let Some(cell) = cell {
                    cell.put(Buffer {
                        timestamp: start,
                        snapshot: None,
                        data: Mencoder::to_msgpack(&stats.history).unwrap(),
                    });
                }
            }
        }

        sleep(Duration::milliseconds(2000 - time_ms() as i64 % 2000));
    }
}
