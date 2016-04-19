use std::sync::{RwLock, Mutex, Condvar};
use std::fs::{File, rename, remove_file, read_dir};
use std::os::unix::fs::symlink;
use std::io::Write;
use std::str::FromStr;
use std::path::Path;

use regex::Regex;

use super::stats::Stats;
use super::scan::time_ms;
use super::deps::{Dependencies, LockedDeps};


pub struct MetricBuffer {
    pub timestamp: u64,
    pub snapshot: Option<String>,
    pub data: Box<[u8]>,
}

pub enum Task {
    Metrics(MetricBuffer),
    Peers(Box<[u8]>),
}

#[derive(Default, RustcEncodable, Clone, Copy)]
pub struct StorageStats {
    pub time: u64,
    pub timestamp: u64,
    pub duration: u32,
    pub size: usize,
}

struct Items {
    metrics: Option<MetricBuffer>,
    peers: Option<Box<[u8]>>,
}

pub struct Storage {
    value: Mutex<Items>,
    cond: Condvar,
}

impl Storage {
    pub fn new() -> Storage {
        Storage {
            value: Mutex::new(Items {
                metrics: None,
                peers: None,
            }),
            cond: Condvar::new(),
        }
    }
    pub fn store_metrics(&self, value: MetricBuffer) {
        let mut lock = self.value.lock().unwrap();
        lock.metrics = Some(value);
        self.cond.notify_all();
    }
    pub fn get(&self) -> Task {
        let mut lock = self.value.lock().expect("storage lock");
        loop {
            if let Some(val) = lock.peers.take() {
                return Task::Peers(val);
            }
            if let Some(val) = lock.metrics.take() {
                return Task::Metrics(val);
            }
            lock = self.cond.wait(lock).expect("storage lock");
        }
    }
}

pub fn store_metrics(path: &Path, buf: MetricBuffer, stats: &RwLock<Stats>) {
    let tmp = path.join("current.tmp");
    let tmplink = path.join("current.tmp.link");
    let current = path.join("current.cbor");
    let file_re = Regex::new(r#"^hourly-(\d+).cbor$"#).unwrap();
    let start_time = time_ms();
    File::create(&tmp)
    .and_then(|mut f| f.write_all(&buf.data))
    .and_then(|()| {
        if let Some(ref filename) = buf.snapshot {
            let filename = path.join(filename).with_extension("cbor");
            try!(symlink(&filename, &tmplink));
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
            stats.storage.duration = dur;
            stats.storage.time = time;
            stats.storage.timestamp = buf.timestamp;
            stats.storage.size = buf.data.len();
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

pub fn storage_loop(deps: Dependencies, path: &Path) {
    let cell: &Storage = &*deps.copy();
    let stats: &RwLock<Stats> = &*deps.copy();
    loop {
        match cell.get() {
            Task::Metrics(buf) => store_metrics(path, buf, stats),
            Task::Peers(buf) => unimplemented!(),
        }
    }
}
