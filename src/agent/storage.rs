use std::sync::{RwLock};
use std::fs::{File, rename};
use std::io::Write;
use super::util::Cell;
use super::stats::Stats;
use super::scan::time_ms;

pub struct Buffer {
    pub timestamp: u64,
    pub snapshot: Option<String>,
    pub data: Vec<u8>,
}


pub fn storage_loop(cell: &Cell<Buffer>, path: &Path, stats: &RwLock<Stats>) {
    loop {
        let buf = cell.get();
        let start_time = time_ms();
        let filename = path
            .join(buf.snapshot.as_ref().map(|x| &x[..]).unwrap_or("current"));
        File::create(&filename.with_extension("tmp"))
        .and_then(|mut f| f.write_all(&buf.data))
        .and_then(|()| {
            rename(&filename.with_extension("tmp"),
                   &filename.with_extension("msgpack"))
        })
        .map(|()| {
            let time = time_ms();
            let dur = (time - start_time) as u32;
            debug!("Stored {:?}: {} bytes in {} ms",
                &filename, buf.data.len(), dur);
            if let Ok(mut stats) = stats.write() {
                stats.store_duration = dur;
                stats.store_time = time;
                stats.store_timestamp = buf.timestamp;
                stats.store_size = buf.data.len();
            }
        })
        .map_err(|e| error!("Error storing snapshot {:?}: {}", filename, e));
    }
}
