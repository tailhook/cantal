use std::str::FromStr;
use std::io::{Read};
use std::str::from_utf8;
use std::fs::{File, read_dir};
use libc;

use cantal::itertools::{NextValue, NextStr, words};
use super::Tip;
use super::super::stats::Key;

pub type Pid = u32;

pub struct ReadCache {
    tick: u32,
}

#[derive(RustcEncodable)]
pub struct MinimalProcess {
    pub pid: Pid,
    pub ppid: Pid,
    pub name: String,
    pub state: char,
    pub vsize: u64,
    pub rss: u64,
    pub num_threads: u32,
    pub start_time: u64,
    pub user_time: u32,
    pub system_time: u32,
    pub child_user_time: u32,
    pub child_system_time: u32,
    pub cmdline: String,
    pub read_bytes: u64,
    pub write_bytes: u64,
}

fn page_size() -> usize {
    // TODO(tailhook) use env::page_size when that's stabilized
    return 4096;
}

fn read_process(cache: &mut ReadCache, pid: Pid)
    -> Result<MinimalProcess, ()>
{
    let cmdline = {
        let mut buf = [0u8; 4096];
        let bytes = try!(File::open(&format!("/proc/{}/cmdline", pid))
            .and_then(|mut f| f.read(&mut buf))
            .map_err(|_| debug!("Can't read cmdline file")));
        // Command-line may be non-full, but we don't care
        String::from_utf8_lossy(&buf[..bytes]).to_string()
    };

    let mut buf = [0u8; 2048];
    let bytes = try!(File::open(&format!("/proc/{}/stat", pid))
        .and_then(|mut f| f.read(&mut buf))
        .map_err(|e| debug!("Can't read stat file: {}", e)));
    if bytes == 2048 {
        error!("Stat line too long");
        return Err(());
    }

    let buf = &buf[..bytes];
    let name_start = try!(buf.iter().position(|x| x == &b'(').ok_or(()));
    // Since there might be brackets in the name itself we should use last
    // closing paren
    let name_end = try!(buf.iter().rposition(|x| x == &b')').ok_or(()));
    let name = try!(from_utf8(&buf[name_start+1..name_end])
        .map_err(|e| debug!("Can't decode stat file: {}", e)))
        .to_string();

    let stat_line = try!(from_utf8(&buf[name_end+1..])
        .map_err(|e| debug!("Can't decode stat file: {}", e)));
    let mut words = words(&stat_line);

    let mut buf = String::with_capacity(512);
    try!(File::open(&format!("/proc/{}/io", pid))
        .and_then(|mut f| f.read_to_string(&mut buf))
        .map_err(|e| debug!("Can't read io file: {}", e)));
    let mut read_bytes = None;
    let mut write_bytes = None;
    for line in buf.split('\n') {
        let mut pieces = line.split_whitespace();
        let name = pieces.next();
        if name == Some("read_bytes:") {
            read_bytes = pieces.next().and_then(|x| FromStr::from_str(x).ok());
        } else if name == Some("write_bytes:") {
            write_bytes = pieces.next().and_then(|x| FromStr::from_str(x).ok());
        }
    }
    let read_bytes = try!(read_bytes.ok_or(())
        .map_err(|_| error!("Can't parse /proc/{}/io", pid)));
    let write_bytes = try!(write_bytes.ok_or(())
        .map_err(|_| error!("Can't parse /proc/{}/io", pid)));

    return Ok(MinimalProcess {
        pid: pid,
        name: name,
        state: try!(words.next_str()).chars().next().unwrap_or('-'),
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
        cmdline: cmdline,
        read_bytes: read_bytes,
        write_bytes: write_bytes,
    });
}

pub fn read(cache: &mut ReadCache) -> Vec<MinimalProcess> {
    read_dir("/proc")
    .map_err(|e| error!("Error listing /proc: {}", e))
    .map(|lst| lst
        .filter_map(|x| x.ok())
        .filter_map(|x| x.path().file_name()
                         .and_then(|x| x.to_str())
                         .and_then(|x| FromStr::from_str(x).ok()))
        .filter_map(|x| read_process(cache, x).ok())
        .collect())
    .unwrap_or(Vec::new())
}

impl ReadCache {
    pub fn new() -> ReadCache {
        ReadCache {
            tick: unsafe {
                libc::sysconf(libc::consts::os::sysconf::_SC_CLK_TCK) as u32
            },
        }
    }
}

pub fn write_tip(tip: &mut Tip, processes: &Vec<MinimalProcess>) {
    use cantal::Value::*;

    for p in processes {
        let pid = p.pid.to_string();
        tip.add(Key::pairs(&[
            ("pid", &pid[..]),
            ("metric", "vsize"),
            ]),
            Integer(p.vsize as i64));
        tip.add(Key::pairs(&[
            ("pid", &pid[..]),
            ("metric", "rss"),
            ]),
            Integer(p.rss as i64));
        tip.add(Key::pairs(&[
            ("pid", &pid[..]),
            ("metric", "num_threads"),
            ]),
            Integer(p.num_threads as i64));
        tip.add(Key::pairs(&[
            ("pid", &pid[..]),
            ("metric", "user_time"),
            ]),
            Counter(p.user_time as u64));
        tip.add(Key::pairs(&[
            ("pid", &pid[..]),
            ("metric", "system_time"),
            ]),
            Counter(p.system_time as u64));
        tip.add(Key::pairs(&[
            ("pid", &pid[..]),
            ("metric", "read_bytes"),
            ]),
            Counter(p.read_bytes));
        tip.add(Key::pairs(&[
            ("pid", &pid[..]),
            ("metric", "write_bytes"),
            ]),
            Counter(p.write_bytes));
        // TODO(tailhook) io
        // TODO(tailhook) FDSize
    }
}
