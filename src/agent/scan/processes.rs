use std::str::FromStr;
use std::env::page_size;
use std::ffi::OsStr;
use std::os::unix::prelude::OsStrExt;
use std::io::{BufReader, BufRead, Read};
use std::str::from_utf8;
use std::hash::{Hash};
use std::path::{Path, PathBuf};
use std::fs::{File, read_dir};
use std::collections::{HashMap, BTreeMap};
use std::collections::hash_map::Entry::{Occupied, Vacant};
use libc;

use cantal::Metadata;
use cantal::itertools::{NextValue, NextStr};
use cantal::iotools::{ReadHostBytes};

type Pid = u32;

pub struct ReadCache {
    metadata: HashMap<PathBuf, Metadata>,
    tick: u32,
}

#[derive(Encodable)]
pub struct MinimalProcess {
    pid: Pid,
    ppid: Pid,
    name: String,
    state: char,
    vsize: u64,
    rss: u64,
    num_threads: u32,
    start_time: u64,
    user_time: u32,
    system_time: u32,
    child_user_time: u32,
    child_system_time: u32,
    cmdline: String,
}

/*
struct Group {
    pids: Vec<u32>,
    path: Path,
}
*/

#[derive(Encodable, Default)]
pub struct Processes {
    pub all: Vec<MinimalProcess>,
}

fn get_env_var(pid: u32) -> Option<PathBuf> {
    File::open(&format!("/proc/{}/environ", pid))
    .map(|f| BufReader::new(f))
    .and_then(|mut f| {
        loop {
            let mut line = Vec::with_capacity(4096);
            try!(f.read_until(0, &mut line));
            if line.len() == 0 {
                return Ok(None);
            };
            if line.starts_with(b"CANTAL_PATH=") {
                return Ok(Some(PathBuf::new(<OsStr as OsStrExt>::from_bytes(
                    &line["CANTAL_PATH=".len()..line.len()-1]))));
            }
        }
    })
    .map_err(|e| debug!("Can't read environ file: {}", e))
    .ok()
    .and_then(|opt| opt)
}

fn read_process(cache: &mut ReadCache, pid: Pid)
    -> Result<MinimalProcess, ()>
{
    let cmdline = try!(File::open(&format!("/proc/{}/cmdline", pid))
        .and_then(|mut f| f.read_chunk(4096))
        .map_err(|e| debug!("Can't read cmdline file")));
    // Command-line may be non-full, but we don't care
    let cmdline = String::from_utf8_lossy(cmdline.as_slice());

    let buf = try!(File::open(&format!("/proc/{}/stat", pid))
        .and_then(|mut f| f.read_chunk(4096))
        .map_err(|e| debug!("Can't read stat file: {}", e)));
    if buf.len() >= 4096 {
        error!("Stat line too long");
        return Err(());
    }

    let name_start = try!(buf.position_elem(&b'(').ok_or(()));
    // Since there might be brackets in the name itself we should use last
    // closing paren
    let name_end = try!(buf.rposition_elem(&b')').ok_or(()));
    let name = try!(from_utf8(&buf[name_start+1..name_end])
        .map_err(|e| debug!("Can't decode stat file: {}", e)))
        .to_string();

    let mut words = try!(from_utf8(&buf[name_end+1..])
        .map_err(|e| debug!("Can't decode stat file: {}", e)))
        .words();


    return Ok(MinimalProcess {
        pid: pid,
        name: name,
        state: try!(words.next_str()).char_at(0),
        ppid: try!(words.next_value()),
        user_time: try!(words.nth_value(9)),
        system_time: try!(words.next_value()),
        child_user_time: try!(words.next_value()),
        child_system_time: try!(words.next_value()),
        num_threads: try!(words.nth_value(2)),
        start_time: {
            let stime: u64 = try!(words.nth_value(1));
            (stime * 1000) / cache.tick as u64 },
        vsize: try!(words.next_value()),
        rss: {
            let rss: u64 = try!(words.next_value());
            rss * page_size() as u64},
        cmdline: cmdline.to_string(),
    });
    /*
        cantal_path.map(|path| {
            let meta = match Metadata::read(&path.with_extension("meta")) {
                Ok(meta) => meta,
                Err(e) => {
                    warn!("Error parsing metadata {:?}: {}", path, e);
                    return;
                }
            };
            let data = match meta.read_data(&path.with_extension("values")) {
                Ok(data) => data,
                Err(e) => {
                    warn!("Error parsing data {:?}: {}", path, e);
                    return;
                }
            };
            for &(ref descr, ref item) in data.iter() {
                println!("{} {} {:?}", pid, descr.textname, item);
            }
        });
    */
}

fn tree_collect<K: Hash + Eq, V, I: Iterator<Item=(K, V)>>(mut iter: I)
    -> HashMap<K, Vec<V>>
{
    let mut result = HashMap::new();
    for (k, v) in iter {
        if let Some(vec) = result.get_mut(&k) {
            let mut val: &mut Vec<V> = vec;
            val.push(v);
            continue;
        }
        result.insert(k, vec!(v));
    }
    return result;
}

fn read_processes(cache: &mut ReadCache) -> Result<Vec<MinimalProcess>, ()> {
    read_dir("/proc")
    .map_err(|e| error!("Error listing /proc: {}", e))
    .map(|lst| lst
        .filter_map(|x| x.ok())
        .filter_map(|x| x.path().file_name()
                         .and_then(|x| x.to_str())
                         .and_then(|x| FromStr::from_str(x).ok()))
        .filter_map(|x| read_process(cache, x).ok())
        .collect())
}

pub fn read(cache: &mut ReadCache) -> Processes {
    let processes = read_processes(cache).unwrap_or(Vec::new());
    {
        let children: HashMap<u32, Vec<&MinimalProcess>>;
        children = tree_collect(processes.iter().map(|p| (p.ppid, p)));
    }
    return Processes {
        all: processes,
    };
}

impl ReadCache {
    pub fn new() -> ReadCache {
        ReadCache {
            metadata: HashMap::new(),
            tick: unsafe {
                libc::sysconf(libc::consts::os::sysconf::_SC_CLK_TCK) as u32
            },
        }
    }
}
