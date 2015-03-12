use std::sync::{RwLock};
use std::fs::{File, rename, soft_link};
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
    let tmp = path.join("current.tmp");
    let tmplink = path.join("current.tmp.link");
    let current = path.join("current.msgpack");
    loop {
        let buf = cell.get();
        let start_time = time_ms();
        File::create(&tmp)
        .and_then(|mut f| f.write_all(&buf.data))
        .and_then(|()| {
            if let Some(ref filename) = buf.snapshot {
                let filename = path.join(filename).with_extension("msgpack");
                try!(soft_link(&filename, &tmplink));
                try!(rename(&tmp, &filename));
                rename(&tmplink, &current)
            } else {
                rename(&tmp, &current)
            }
        })
        .map(|()| {
            let time = time_ms();
            let dur = (time - start_time) as u32;
            debug!("Stored {:?}: {} bytes in {} ms",
                &buf.snapshot, buf.data.len(), dur);
            if let Ok(mut stats) = stats.write() {
                stats.store_duration = dur;
                stats.store_time = time;
                stats.store_timestamp = buf.timestamp;
                stats.store_size = buf.data.len();
            }
        })
        .map_err(|e| error!("Error storing snapshot: {}", e))
        .ok();
    }
}
