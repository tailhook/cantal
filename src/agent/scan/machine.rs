use std::default::Default;
use std::str::FromStr;
use std::io::fs::File;
use std::io::BufferedReader;


use super::{time_ms};

#[derive(Default, Encodable)]
pub struct Cpu {
    pub user: Option<u64>,
    pub nice: Option<u64>,
    pub system: Option<u64>,
    pub idle: Option<u64>,
    pub iowait: Option<u64>,
    pub irq: Option<u64>,
    pub softirq: Option<u64>,
    pub steal: Option<u64>,
    pub guest: Option<u64>,
    pub guest_nice: Option<u64>,
}

#[derive(Default, Encodable)]
pub struct Memory {
    pub mem_total: Option<u64>,
    pub mem_free: Option<u64>,
    pub mem_available: Option<u64>,
    pub buffers: Option<u64>,
    pub cached: Option<u64>,
    pub swap_cached: Option<u64>,
    pub active: Option<u64>,
    pub inactive: Option<u64>,
    pub unevictable: Option<u64>,
    pub mlocked: Option<u64>,
    pub swap_total: Option<u64>,
    pub swap_free: Option<u64>,
    pub dirty: Option<u64>,
    pub writeback: Option<u64>,
    pub commit_limit: Option<u64>,
    pub committed_as: Option<u64>,
}

#[derive(Default, Encodable)]
pub struct MachineStats {
    pub timestamp: u64,
    pub uptime: Option<f64>,
    pub idle_time: Option<f64>,
    pub load_avg_1min: Option<f32>,
    pub load_avg_5min: Option<f32>,
    pub load_avg_15min: Option<f32>,
    pub proc_runnable: Option<u32>,
    pub proc_total: Option<u32>,
    pub last_pid: Option<u32>,
    pub cpu_total: Option<Cpu>,
    pub memory: Memory,
    pub boot_time: Option<u64>,
}

pub fn read() -> MachineStats {
    let mut result: MachineStats = Default::default();

    File::open(&Path::new("/proc/uptime"))
        .and_then(|mut f| f.read_to_string())
        .map(|buf| {
            let mut pieces = buf.words();
            result.uptime = pieces.next().and_then(FromStr::from_str);
            result.idle_time = pieces.next().and_then(FromStr::from_str);
        }).ok();

    File::open(&Path::new("/proc/loadavg"))
        .and_then(|mut f| f.read_to_string())
        .map(|buf| {
            let mut pieces = buf.words();
            result.load_avg_1min = pieces.next().and_then(FromStr::from_str);
            result.load_avg_5min = pieces.next().and_then(FromStr::from_str);
            result.load_avg_15min = pieces.next().and_then(FromStr::from_str);
            let mut proc_pieces = pieces.next()
                .map(|x| x.splitn(1, '/'))
                .map(|mut p| {
                    result.proc_runnable = p.next().and_then(FromStr::from_str);
                    result.proc_total = p.next().and_then(FromStr::from_str);
                });
            result.last_pid = pieces.next().and_then(FromStr::from_str);
        }).ok();

    File::open(&Path::new("/proc/stat"))
        .and_then(|f| {
            let mut f = BufferedReader::new(f);
            for line in f.lines() {
                let line = try!(line);
                if line.starts_with("cpu ") {
                    let mut pieces = line.words();
                    result.cpu_total = Some(Cpu {
                        user: pieces.nth(1).and_then(FromStr::from_str),
                        nice: pieces.next().and_then(FromStr::from_str),
                        system: pieces.next().and_then(FromStr::from_str),
                        idle: pieces.next().and_then(FromStr::from_str),
                        iowait: pieces.next().and_then(FromStr::from_str),
                        irq: pieces.next().and_then(FromStr::from_str),
                        softirq: pieces.next().and_then(FromStr::from_str),
                        steal: pieces.next().and_then(FromStr::from_str),
                        guest: pieces.next().and_then(FromStr::from_str),
                        guest_nice: pieces.next().and_then(FromStr::from_str),
                    });
                } else if line.starts_with("btime ") {
                    result.boot_time = FromStr::from_str(line[6..].trim());
                }
            }
            Ok(())
        }).ok();

    File::open(&Path::new("/proc/meminfo"))
        .and_then(|f| {
            let mut f = BufferedReader::new(f);
            for line in f.lines() {
                let line = try!(line);
                let mut pieces = line.words();
                let ptr = match pieces.next() {
                    Some("MemTotal:") => &mut result.memory.mem_total,
                    Some("MemFree:") => &mut result.memory.mem_free,
                    Some("MemAvailable:") => &mut result.memory.mem_available,
                    Some("Buffers:") => &mut result.memory.buffers,
                    Some("Cached:") => &mut result.memory.cached,
                    Some("SwapCached:") => &mut result.memory.swap_cached,
                    Some("Active:") => &mut result.memory.active,
                    Some("Inactive:") => &mut result.memory.inactive,
                    Some("Unevictable:") => &mut result.memory.unevictable,
                    Some("Mlocked:") => &mut result.memory.mlocked,
                    Some("SwapTotal:") => &mut result.memory.swap_total,
                    Some("SwapFree:") => &mut result.memory.swap_free,
                    Some("Dirty:") => &mut result.memory.dirty,
                    Some("Writeback:") => &mut result.memory.writeback,
                    Some("CommitLimit:") => &mut result.memory.commit_limit,
                    Some("Committed_AS:") => &mut result.memory.committed_as,
                    _ => continue,
                };
                let val = match pieces.next() {
                    Some(val) => val,
                    None => continue,
                };
                let mult = match pieces.next() {
                    Some("kB") => 1024,
                    Some(x) => {
                        debug!("Unknown memory unit {:?}", x);
                        continue;
                    }
                    None => continue,
                };
                *ptr = FromStr::from_str(val).map(|x: u64| x * mult);
            }
            Ok(())
        }).ok();

    result.timestamp = time_ms();
    result
}
