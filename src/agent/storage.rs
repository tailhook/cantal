use std::sync::{RwLock};
use std::fs::{File, rename, soft_link, remove_file, read_dir};
use std::io::Write;
use std::str::FromStr;
use std::path::Path;

use regex::Regex;

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
    let file_re = Regex::new(r#"^hourly-(\d+).msgpack$"#).unwrap();
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
        let cut_off = start_time / 3_600_000 - 36;  // keep 36 hours
        read_dir(&path).map(|iter| for item in iter {
            item.map(|entry| {
                entry.path().file_name()
                .and_then(|x| x.to_str())
                .and_then(|fname| file_re.captures(fname))
                .and_then(|c| c.at(1))
                .and_then(|x| FromStr::from_str(x).ok())
                .map(|x: u64| {
                    if x < cut_off {
                        remove_file(entry.path())
                        .map_err(|e| error!("Can't remove old file {:?}: {}",
                                            entry.path(), e))
                        .ok();
                    }
                });
            }).ok();
        }).map_err(|e| error!("Can't read dir: {}", e)).ok();
    }
}
