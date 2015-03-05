use std::hash::{Hash, Hasher};
use std::default::Default;
use std::collections::BTreeMap;
use super::scan::time_ms;
use super::scan;
use serialize::json::Json;
use msgpack::Value as Mpack;
use msgpack::Encoder as Mencoder;

pub struct Stats {
    pub startup_time: u64,
    pub scan_time: u64,
    pub boot_time: Option<u64>,
    pub tip: scan::Tip,
    pub processes: scan::processes::Processes,
}

impl Stats {
    pub fn new() -> Stats {
        return Stats {
            startup_time: time_ms(),
            scan_time: 0,
            boot_time: None,
            tip: scan::Tip::new(),
            processes: Default::default(),
        };
    }
}

#[derive(Hash, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Key(BTreeMap<String, String>);

impl Key {
    pub fn from_json(json: &Json) -> Result<Key, ()> {
        if let &Json::Object(ref obj) = json {
            let mut key = BTreeMap::new();
            for (k, v) in obj {
                match v {
                    &Json::String(ref val) => {
                        key.insert(k.clone(), val.clone());
                    }
                    _ => return Err(()),
                }
            }
            return Ok(Key(key));
        } else {
            return Err(());
        }
    }
    pub fn from_pair(key: &str, val: &str) -> Key {
        let mut res = BTreeMap::new();
        res.insert(key.to_string(), val.to_string());
        return Key(res);
    }
    pub fn metric(metric: &str) -> Key {
        return Key::from_pair("metric", metric);
    }
}


