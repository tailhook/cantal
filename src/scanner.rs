use std::cmp::min;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use libc::usleep;
use probor::{Encoder, Encodable};

use super::stats::Stats;
use super::scan::Tip;
use super::scan::machine;
use super::scan::processes;
use super::scan::connections;
use super::scan::values;
use super::scan::time_ms;
use super::scan::cgroups;
use super::deps::{Dependencies, LockedDeps};
use cantal::Value;
use history::VersionInfo;
use storage::{Storage, MetricBuffer};

use incoming::Incoming;


const SNAPSHOT_INTERVAL: u64 = 60000;

fn to_ms(dur: Duration) -> u64 {
    return dur.as_secs() * 1000 + dur.subsec_nanos() as u64 / 1000_000;
}

pub fn scan_loop(deps: Dependencies, interval: u32, backlog_time: Duration,
    incoming: &Incoming)
{
    let stats: &RwLock<Stats> = &*deps.copy();
    let storage = deps.get::<Arc<Storage>>().map(|x| &*x);
    let mut last_store = time_ms();
    let mut last_hourly = last_store / 3_600_000;
    let mut process_cache = processes::ReadCache::new();
    let mut values_cache = values::ReadCache::new();
    let mut last_buffer_size = 16 << 10;
    loop {
        let start = time_ms();
        let mut tip = Tip::new();

        let boot_time = machine::read(&mut tip);

        let cgroups = cgroups::read();
        let processes = processes::read(&mut process_cache, &cgroups);
        let connections = connections::read();
        processes::write_tip(&mut tip, &processes, &cgroups);
        values::read(&mut tip, &mut values_cache, &processes, &cgroups);

        let scan_duration = (time_ms() - start) as u32;

        if let Ok(ref mut stats) = stats.write() {
            stats.scan_duration = scan_duration;
            debug!("Got {} values and {} processes in {} ms",
                tip.map.len(), processes.len(), scan_duration);

            // TODO(tailhook) use drain-style iterator and push to both
            // at once, so we don't need clone (each metric)
            stats.history.tip.push((start, scan_duration), tip.map.iter()
                .filter(|&(_, v)| matches!(v, &Value::State(_))));
            stats.history.fine.push((start, scan_duration), tip.map.iter()
                .filter(|&(_, v)| !matches!(v, &Value::State(_))));

            stats.last_scan = start;
            stats.boot_time = boot_time.or(stats.boot_time);
            stats.processes = processes;
            stats.connections = connections;

            if start - last_store > SNAPSHOT_INTERVAL {
                last_store = start;

                // Don't store tip values older than a minute
                stats.history.tip.truncate_by_time(
                    start - min(to_ms(backlog_time), 60000));

                let mut snapshot = None;
                if backlog_time > Duration::new(3600, 0) {
                    let hourly = start / 3_600_000;
                    if hourly > last_hourly {
                        stats.history.fine.truncate_by_time(
                            start - to_ms(backlog_time));
                        snapshot = Some(format!("hourly-{}", hourly));
                        last_hourly = hourly;
                    }
                } else {
                    // Never store hourly snapshot if backlog time less than
                    // an hour
                    stats.history.fine.truncate_by_time(
                        start - to_ms(backlog_time));
                }

                // Preallocate a buffer of same size as previous one, since
                // it's expected about same size. But add few kb, so that
                // 99% of the time no further allocations are necessary
                let mut enc = Encoder::new(
                    Vec::with_capacity(last_buffer_size + 16384));
                VersionInfo::current().encode(&mut enc)
                .and_then(|()| stats.history.encode(&mut enc))
                .map(|()| {
                    let buf = enc.into_writer();
                    last_buffer_size = buf.len();

                    if let Some(storage) = storage {
                        storage.store_metrics(MetricBuffer {
                            timestamp: start,
                            snapshot: snapshot,
                            data: buf.into_boxed_slice(),
                        });
                    }
                }).map_err(|e| error!("Can't encode history: {}", e)).ok();
            }
        }
        incoming.trigger_status_change();
        incoming.trigger_scan_change();

        unsafe { usleep(((interval as i64 - time_ms() as i64 % interval as i64)*1000) as u32) };
    }
}
