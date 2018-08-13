use std::collections::{HashMap, HashSet};

use history::Key;
use incoming::Connection;


#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Filter {
    pub exact_key: Key,
}

#[derive(Debug)]
pub struct Conn {
    pub filters: HashMap<i32, Filter>,
}

impl Conn {
    pub fn new() -> Conn {
        Conn {
            filters: HashMap::new(),
        }
    }
}

#[derive(Debug)]
pub struct Global {
    // TODO(tailhook) remove me, not needed for now
    pub keys: HashMap<Key, HashSet<Connection>>,
}

impl Global {
    pub fn new() -> Global {
        Global {
            keys: HashMap::new(),
        }
    }
    /// This method returns key as it's stored in the map to remove
    /// duplicates of the underlying key data
    pub fn track_key(&mut self, key: &Key, connection: &Connection)
        -> Key
    {
        let e = self.keys.entry(key.clone());
        let key = e.key().clone();
        let subs = e.or_insert_with(HashSet::new);
        subs.insert(connection.clone());
        return key;
    }
    pub fn untrack_key(&mut self, key: &Key, connection: &Connection) {
        let delete_key = if let Some(conns) = self.keys.get_mut(key) {
            conns.remove(connection);
            conns.len() == 0
        } else {
            false
        };
        if delete_key {
            self.keys.remove(key);
        }
    }
}
