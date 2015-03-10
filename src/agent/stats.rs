use std::hash::{Hash, Hasher};
use std::default::Default;
use std::collections::BTreeMap;

use serialize::json::{Json, ToJson};

use super::scan::time_ms;
use super::scan;
use super::history::History;

#[derive(Encodable)]
pub struct Stats {
    pub startup_time: u64,
    pub scan_time: u64,
    pub boot_time: Option<u64>,
    pub history: History,
    pub processes: Vec<scan::processes::MinimalProcess>,
}

impl Stats {
    pub fn new() -> Stats {
        return Stats {
            startup_time: time_ms(),
            scan_time: 0,
            boot_time: None,
            history: History::new(),
            processes: Default::default(),
        };
    }
}

#[derive(Hash, PartialEq, Eq, PartialOrd, Ord, Debug, Encodable, Decodable)]
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


