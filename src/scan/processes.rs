use std::str::FromStr;
use std::sync::Arc;
use std::io::{Read, BufReader, BufRead};
use std::str::from_utf8;
use std::fs::{File, read_dir};
use std::collections::HashMap;

use libc;

use cantal::itertools::{NextValue, NextStr};
use frontend::graphql::ContextRef;
use history::Key;
use scan::cgroups::CGroups;
use super::Tip;

pub type Pid = u32;

pub struct ReadCache {
    tick: u32,
    boot_time: u64,
    page_size: usize,
}

#[derive(Serialize, Debug)]
pub struct MinimalProcess {
    pub pid: Pid,
    pub ppid: Pid,
    pub uid: u32,
    pub gid: u32,
    pub name: String,
    pub state: char,
    pub vsize: u64,
    pub rss: u64,
    pub swap: u64,
    pub num_threads: u32,
    pub start_time: u64,
    pub start_timestamp: u64,
    pub user_time: u32,
    pub system_time: u32,
    pub child_user_time: u32,
    pub child_system_time: u32,
    pub cmdline: String,
    pub read_bytes: u64,
    pub write_bytes: u64,
    pub cgroup: Option<Arc<String>>,
}

graphql_object!(<'a> &'a MinimalProcess: ContextRef<'a> as "MinimalProcess" |&self| {
    field pid() -> i32 { self.pid as i32 }
    field ppid() -> i32 { self.pid as i32 }
    field rss() -> f64 { self.rss as f64 }
    field swap() -> f64 { self.swap as f64 }
    field start_timestamp() -> f64 { self.start_timestamp as f64 }
});

fn parse_io(pid: Pid) -> Result<(u64, u64), ()> {
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
    Ok((read_bytes, write_bytes))
}

fn parse_status(pid: Pid) -> Result<(u32, u32, u64), ()> {
    let buf = BufReader::new(try!(File::open(&format!("/proc/{}/status", pid))
        .map_err(|e| debug!("Can't read io file: {}", e))));
    _parse_status(buf, pid)
}
fn _parse_status<R: BufRead>(buf: R, pid: Pid) -> Result<(u32, u32, u64), ()> {
    let mut uid = None;
    let mut gid = None;
    let mut swap = None;
    for line in buf.lines() {
        let line = try!(line
            .map_err(|e| debug!("Can't read io file: {}", e)));
        let mut pair = line.split(':');
        match (pair.next(), pair.next()) {
            (Some("Uid"), Some(v)) => {
                // The line is:
                //  Uid:    1000    1000    1000    1000
                // We pick first field -- a real user id
                uid = v.split_whitespace().next()
                    .and_then(|x| x.parse().ok());
            }
            (Some("Gid"), Some(v)) => {
                // The line is:
                //  Gid:    100     100     100     100
                // We pick first field -- a real group id
                gid = v.split_whitespace().next()
                    .and_then(|x| x.parse().ok());
            }
            (Some("VmSwap"), Some(v)) => {
                let mut parts = v.split_whitespace();
                let maybe_kb = parts.next()
                    .and_then(|x| x.parse().ok());
                if parts.next() == Some("kB") {
                    swap = maybe_kb.map(|x: u64| x * 1024);
                } // any other units possible?
            }
            _ => {}
        }
        if uid.is_some() && gid.is_some() && swap.is_some() { break; }
    }
    let uid = try!(uid.ok_or(())
        .map_err(|_| error!("Can't parse /proc/{}/status", pid)));
    let gid = try!(gid.ok_or(())
        .map_err(|_| error!("Can't parse /proc/{}/status", pid)));
    // kernel threads do not have swap info, as well as all processes on older
    // kernels
    let swap = swap.unwrap_or(0);
    Ok((uid, gid, swap))
}

fn read_process(cache: &mut ReadCache, cgroup: Option<&Arc<String>>, pid: Pid)
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
    let mut words = stat_line.split_whitespace();

    let (read_bytes, write_bytes) = try!(parse_io(pid));
    let (uid, gid, swap) = try!(parse_status(pid));

    let state = words.next_str()?.chars().next().unwrap_or('-');
    let ppid = words.next_value()?;
    let user_time = words.nth_value(9)?;
    let system_time = words.next_value()?;
    let child_user_time = words.next_value()?;
    let child_system_time = words.next_value()?;
    let num_threads = words.nth_value(2)?;
    let stime: u64 = words.nth_value(1)?;
    let start_time = (stime * 1000) / cache.tick as u64;
    let start_timestamp = cache.boot_time + start_time;
    let vsize = words.next_value()?;
    let rss = words.next_value::<u64>()? * cache.page_size as u64;

    return Ok(MinimalProcess {
        pid, ppid, uid, gid,
        name, state,
        user_time, system_time, child_user_time, child_system_time,
        num_threads,
        start_time, start_timestamp,
        vsize, rss, swap,
        cmdline,
        read_bytes, write_bytes,
        cgroup: cgroup.map(|x| x.clone()),
    });
}

pub fn read(cache: &mut ReadCache, cgroups: &HashMap<Pid, Arc<String>>)
    -> Vec<MinimalProcess>
{
    read_dir("/proc")
    .map_err(|e| error!("Error listing /proc: {}", e))
    .map(|lst| lst
        .filter_map(|x| x.ok())
        .filter_map(|x| x.path().file_name()
                         .and_then(|x| x.to_str())
                         .and_then(|x| FromStr::from_str(x).ok()))
        .filter_map(|x| read_process(cache, cgroups.get(&x), x).ok())
        .collect())
    .unwrap_or(Vec::new())
}

fn boot_time() -> u64 {
    let mut f = BufReader::new(File::open("/proc/stat")
        .expect("/proc/stat must be readable"));
    let mut line = String::with_capacity(100);
    loop {
        line.truncate(0);
        f.read_line(&mut line)
            .expect("/proc/stat must be readable");
        if line.starts_with("btime ") {
            return FromStr::from_str(line[6..].trim())
                .expect("boot time must be parseable");
        }
        if line.len() == 0 {
            panic!("btime must be present in /proc/stat");
        }
    }
}

impl ReadCache {
    pub fn new() -> ReadCache {
        ReadCache {
            tick: unsafe {
                libc::sysconf(libc::_SC_CLK_TCK) as u32
            },
            page_size: unsafe {
                libc::sysconf(libc::_SC_PAGE_SIZE) as usize
            },
            boot_time: boot_time(),
        }
    }
}

fn key(metric: &str, pid: &str, cgroup: Option<&str>) -> Key {
    if let Some(cgrp) = cgroup {
        Key::pairs(&[
            ("cgroup", cgrp),
            ("metric", metric),
            ("pid", pid),
            ])
    } else {
        Key::pairs(&[
            ("metric", metric),
            ("pid", pid),
            ])
    }
}

pub fn write_tip(tip: &mut Tip, processes: &Vec<MinimalProcess>,
    cgroups: &CGroups)
{
    use cantal::Value::*;
    for p in processes {
        let pid = p.pid.to_string();
        let cgroup = cgroups.get(&p.pid).map(|x| &x[..]);
        tip.add(key("vsize", &pid, cgroup),
            Integer(p.vsize as i64));
        tip.add(key("rss", &pid, cgroup),
            Integer(p.rss as i64));
        tip.add(key("num_threads", &pid, cgroup),
            Integer(p.num_threads as i64));
        tip.add(key("user_time", &pid, cgroup),
            Counter(p.user_time as u64));
        tip.add(key("system_time", &pid, cgroup),
            Counter(p.system_time as u64));
        tip.add(key("read_bytes", &pid, cgroup),
            Counter(p.read_bytes));
        tip.add(key("write_bytes", &pid, cgroup),
            Counter(p.write_bytes));
        // TODO(tailhook) FDSize
    }
}

#[cfg(test)]
mod test {
    use super::_parse_status;

    const KTHREAD_DATA: &str = "\
Name:	kthreadd
State:	S (sleeping)
Tgid:	2
Ngid:	0
Pid:	2
PPid:	0
TracerPid:	0
Uid:	0	0	0	0
Gid:	0	0	0	0
FDSize:	64
Groups:
NStgid:	2
NSpid:	2
NSpgid:	0
NSsid:	0
Threads:	1
SigQ:	0/30930
SigPnd:	0000000000000000
ShdPnd:	0000000000000000
SigBlk:	0000000000000000
SigIgn:	ffffffffffffffff
SigCgt:	0000000000000000
CapInh:	0000000000000000
CapPrm:	0000003fffffffff
CapEff:	0000003fffffffff
CapBnd:	0000003fffffffff
CapAmb:	0000000000000000
Seccomp:	0
Cpus_allowed:	f
Cpus_allowed_list:	0-3
Mems_allowed:	00000000,00000001
Mems_allowed_list:	0
voluntary_ctxt_switches:	46253
nonvoluntary_ctxt_switches:	81
";
    const NORMAL_DATA: &str = "\
Name:	cat
State:	R (running)
Tgid:	6283
Ngid:	0
Pid:	6283
PPid:	30904
TracerPid:	0
Uid:	1000	1000	1000	1000
Gid:	100	100	100	100
FDSize:	64
Groups:	1 17 20 26 27 100 131 499
NStgid:	6283
NSpid:	6283
NSpgid:	6283
NSsid:	30904
VmPeak:	  122276 kB
VmSize:	  122276 kB
VmLck:	       0 kB
VmPin:	       0 kB
VmHWM:	     472 kB
VmRSS:	     472 kB
RssAnon:	      76 kB
RssFile:	     396 kB
RssShmem:	       0 kB
VmData:	     440 kB
VmStk:	     140 kB
VmExe:	    1428 kB
VmLib:	    2032 kB
VmPTE:	      56 kB
VmPMD:	      12 kB
VmSwap:	      17 kB
HugetlbPages:	       0 kB
Threads:	1
SigQ:	0/30930
SigPnd:	0000000000000000
ShdPnd:	0000000000000000
SigBlk:	0000000000000000
SigIgn:	0000000000000000
SigCgt:	0000000180000000
CapInh:	0000000000000000
CapPrm:	0000000000000000
CapEff:	0000000000000000
CapBnd:	0000003fffffffff
CapAmb:	0000000000000000
Seccomp:	0
Cpus_allowed:	f
Cpus_allowed_list:	0-3
Mems_allowed:	00000000,00000001
Mems_allowed_list:	0
voluntary_ctxt_switches:	0
nonvoluntary_ctxt_switches:	2
";
    #[test]
    fn parse_normal() {
        assert_eq!(_parse_status(NORMAL_DATA.as_bytes(), 1),
                   Ok((1000, 100, 17408)));
    }

    #[test]
    fn parse_kthread() {
        assert_eq!(_parse_status(KTHREAD_DATA.as_bytes(), 1),
                   Ok((0, 0, 0)));
    }
}
