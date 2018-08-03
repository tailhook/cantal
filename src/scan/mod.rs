use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use history::Key;
use cantal::Value;
use cantal::itertools::NextValue;

pub mod machine;
pub mod processes;
pub mod values;
pub mod cgroups;
pub mod connections;


pub fn time_ms() -> u64 {
    let dur = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    return dur.as_secs() * 1000 + dur.subsec_millis() as u64;
}


pub struct Tip {
    pub map: HashMap<Key, Value>,
}

impl Tip {
    pub fn new() -> Tip {
        return Tip {
            map: HashMap::new(),
        }
    }
    pub fn add(&mut self, key: Key, value: Value) {
        //println!("Adding {:?}: {:?}", key, value);
        self.map.insert(key, value);
    }
    pub fn add_next_float<I:NextValue>(&mut self, key: Key, mut iter: I) {
        if let Ok(x) = iter.next_value() {
            self.add(key, Value::Float(x));
        }
    }
    pub fn add_next_int<I:NextValue>(&mut self, key: Key, mut iter: I) {
        if let Ok(x) = iter.next_value() {
            self.add(key, Value::Integer(x));
        }
    }
    pub fn add_next_cnt<I:NextValue>(&mut self, key: Key, mut iter: I) {
        if let Ok(x) = iter.next_value() {
            self.add(key, Value::Counter(x));
        }
    }
}
