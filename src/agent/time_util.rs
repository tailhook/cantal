use std::time::{SystemTime, UNIX_EPOCH};


pub fn time_ms() -> u64 {
    let tm = SystemTime::now().duration_since(UNIX_EPOCH).expect("time is ok");
    return tm.as_secs() * 1000 + (tm.subsec_nanos() / 1000000) as u64;
}
