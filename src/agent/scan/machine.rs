use std::default::Default;
use std::str::FromStr;
use std::io::fs::File;


use super::{time_ms};


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

    result.timestamp = time_ms();
    result
}
