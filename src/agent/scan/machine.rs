use std::default::Default;
use std::str::FromStr;
use std::fs::File;
use std::io::{BufReader, Read, BufRead};
use cantal::itertools::NextValue;


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
        .and_then(|mut f| {
            let mut buf = String::with_capacity(100);
            f.read_to_string(&mut buf)
            .map(|_| buf)})
        .map(|buf| {
            let mut pieces = buf.words();
            result.uptime = pieces.next_value().ok();
            result.idle_time = pieces.next_value().ok();
        }).ok();

    File::open(&Path::new("/proc/loadavg"))
        .and_then(|mut f| {
            let mut buf = String::with_capacity(100);
            f.read_to_string(&mut buf)
            .map(|_| buf)
        })
        .map(|buf| {
            let mut pieces = buf.words();
            result.load_avg_1min = pieces.next_value().ok();
            result.load_avg_5min = pieces.next_value().ok();
            result.load_avg_15min = pieces.next_value().ok();
            let mut proc_pieces = pieces.next()
                .map(|x| x.splitn(1, '/'))
                .map(|mut p| {
                    result.proc_runnable = p.next_value().ok();
                    result.proc_total = p.next_value().ok();
                });
            result.last_pid = pieces.next_value().ok();
        }).ok();

    File::open(&Path::new("/proc/stat"))
        .and_then(|f| {
            let mut f = BufReader::new(f);
            loop {
                let mut line = String::with_capacity(100);
                try!(f.read_line(&mut line));
                if line.len() == 0 { break; }
                if line.starts_with("cpu ") {
                    let mut pieces = line.words();
                    result.cpu_total = Some(Cpu {
                        user: pieces.nth_value(1).ok(),
                        nice: pieces.next_value().ok(),
                        system: pieces.next_value().ok(),
                        idle: pieces.next_value().ok(),
                        iowait: pieces.next_value().ok(),
                        irq: pieces.next_value().ok(),
                        softirq: pieces.next_value().ok(),
                        steal: pieces.next_value().ok(),
                        guest: pieces.next_value().ok(),
                        guest_nice: pieces.next_value().ok(),
                    });
                } else if line.starts_with("btime ") {
                    result.boot_time = FromStr::from_str(line[6..].trim()).ok();
                }
            }
            Ok(())
        }).ok();

    File::open(&Path::new("/proc/meminfo"))
        .and_then(|f| {
            let mut f = BufReader::new(f);
            loop {
                let mut line = String::with_capacity(50);
                try!(f.read_line(&mut line));
                if line.len() == 0 { break; }
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
                *ptr = FromStr::from_str(val).map(|x: u64| x * mult).ok();
            }
            Ok(())
        }).ok();

    result.timestamp = time_ms();
    result
}
