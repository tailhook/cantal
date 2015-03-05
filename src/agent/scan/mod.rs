use std::ptr;
use std::fmt::String;
use std::collections::HashMap;
use libc;

use super::stats::Key;
use cantal::Value;
use cantal::itertools::NextValue;

pub mod machine;
pub mod processes;

// TODO(tailhook) use some time/date crate

extern {
    fn gettimeofday(tp: *mut libc::timeval, tzp: *mut libc::c_void)
        -> libc::c_int;
}

pub fn time_ms() -> u64 {
    let mut tv = libc::timeval { tv_sec: 0, tv_usec: 0 };
    unsafe { gettimeofday(&mut tv, ptr::null_mut()) };
    return (tv.tv_sec as u64)*1000 +  (tv.tv_usec as u64) / 1000;
}

pub struct Tip {
    map: HashMap<Key, Value>,
}

impl Tip {
    pub fn new() -> Tip {
        return Tip {
            map: HashMap::new(),
        }
    }
    pub fn add(&mut self, key: Key, value: Value) {
        self.map.insert(key, value);
    }
    pub fn add_next_float<I:NextValue>(&mut self, metric: &str, mut iter: I) {
        if let Ok(x) = iter.next_value() {
            self.map.insert(Key::metric(metric), Value::Float(x));
        }
    }
    pub fn add_next_cnt<I:NextValue>(&mut self, metric: &str, mut iter: I) {
        if let Ok(x) = iter.next_value() {
            self.map.insert(Key::metric(metric), Value::Counter(x));
        }
    }
    pub fn get(&self, key: &Key) -> Option<Value> {
        self.map.get(key).map(|x| x.clone())
    }
}
