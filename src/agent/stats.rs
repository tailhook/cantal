use std::hash::{Hasher};
use std::sync::{Arc, RwLock};
use std::default::Default;
use std::collections::BTreeMap;

use rustc_serialize::json::{Json, ToJson, as_json};
use rustc_serialize::{Decodable, Encodable, Encoder, Decoder, json};

use super::scan::time_ms;
use super::scan;
use super::history::History;
use super::storage::StorageStats;
use super::p2p::GossipStats;


pub struct Stats {

    pub startup_time: u64,
    pub last_scan: u64,
    pub scan_duration: u32,
    pub boot_time: Option<u64>,

    pub storage: StorageStats,
    pub history: History,
    pub processes: Vec<scan::processes::MinimalProcess>,
    pub gossip: Arc<RwLock<GossipStats>>,
}

impl Stats {
    pub fn new() -> Stats {
        return Stats {
            startup_time: time_ms(),
            last_scan: 0,
            scan_duration: 0,
            boot_time: None,
            storage: Default::default(),
            gossip: Arc::new(RwLock::new(Default::default())),
            history: History::new(),
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
    pub fn pairs(pairs: &[(&str, &str)]) -> Key {
        let mut res = BTreeMap::new();
        for &(key, val) in pairs.iter() {
            res.insert(key.to_string(), val.to_string());
        }
        return Key(res);
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

// This is needed because rust implementation of Cbor allows only string
// keys
impl Decodable for Key {
    fn decode<D: Decoder>(d: &mut D) -> Result<Key, D::Error> {
        d.read_str().and_then(|x|
            json::decode(&x[..])
            .map_err(|e| d.error(&format!("Error decoding key: {}", e)))
            .map(Key))
    }
}

impl Encodable for Key {
    fn encode<E: Encoder>(&self, e: &mut E) -> Result<(), E::Error> {
        e.emit_str(&format!("{}", as_json(&self.0)))
    }
}

impl ToJson for Key {
    fn to_json(&self) -> Json {
        self.0.to_json()
    }
}

impl<'a> ToJson for &'a Key {
    fn to_json(&self) -> Json {
        self.0.to_json()
    }
}

