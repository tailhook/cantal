use std::hash::{Hash, Hasher};
use std::default::Default;
use std::collections::BTreeMap;

use serialize::json::{Json, ToJson};

use super::scan::time_ms;
use super::scan;
use super::history::History;

#[derive(RustcEncodable)]
pub struct Stats {

    pub startup_time: u64,
    pub scan_duration: u32,
    pub boot_time: Option<u64>,
    pub store_time: u64,
    pub store_timestamp: u64,
    pub store_duration: u32,
    pub store_size: usize,

    pub history: History,
    pub processes: Vec<scan::processes::MinimalProcess>,
}

impl Stats {
    pub fn new() -> Stats {
        return Stats {
            startup_time: time_ms(),
            scan_duration: 0,
            boot_time: None,
            store_time: 0,
            store_timestamp: 0,
            store_duration: 0,
            store_size: 0,
            history: History::new(),
            processes: Default::default(),
        };
    }
}

#[derive(Hash, PartialEq, Eq, PartialOrd, Ord, Debug,
         RustcEncodable, RustcDecodable)]
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
    pub fn pairs(pairs: &[(&str, &str)]) -> Key {
        let mut res = BTreeMap::new();
        for &(key, val) in pairs.iter() {
            res.insert(key.to_string(), val.to_string());
        }
        return Key(res);
    }
    pub fn to_json(&self) -> Json {
        let &Key(ref res) = self;
        return res.to_json();
    }
    pub fn from_pair(key: &str, val: &str) -> Key {
        let mut res = BTreeMap::new();
        res.insert(key.to_string(), val.to_string());
        return Key(res);
    }
    pub fn metric(metric: &str) -> Key {
        return Key::from_pair("metric", metric);
    }
    pub fn add_pair(self, key: &str, val: &str) -> Key {
        let Key(mut res) = self;
        res.insert(key.to_string(), val.to_string());
        return Key(res);
    }
    pub fn get<'x>(&'x self, key: &str) -> Option<&'x str> {
        let &Key(ref map) = self;
        return map.get(key).map(|x| &x[..]);
    }
}


