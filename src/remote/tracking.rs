use std::collections::{HashMap, HashSet};
use std::time::{Instant, Duration};

use history::Key;
use cantal::Value as TipValue;
use incoming::Connection;
use remote::Hostname;
use frontend::last_values::{RemoteMetric};
pub use incoming::tracking::Filter;


const FILTER_TIMEOUT: Duration = Duration::from_secs(5*60);


#[derive(Debug)]
pub struct Tracking {
    pub keys: HashMap<Filter, Info>,
    pub data: HashMap<Key, HashMap<Hostname, TipValue>>,
}

#[derive(Debug)]
pub struct Info {
    connections: HashSet<Connection>,
    timestamp: Instant,
}

impl Tracking {
    pub fn new() -> Tracking {
        Tracking {
            keys: HashMap::new(),
            data: HashMap::new(),
        }
    }
    pub fn add_timed_filter(&mut self, f: &Filter) {
        self.keys.entry(f.clone())
            .and_modify(|x| { x.timestamp = Instant::now() + FILTER_TIMEOUT; })
            .or_insert_with(|| Info {
                connections: HashSet::new(),
                timestamp: Instant::now() + FILTER_TIMEOUT,
            });
        // TODO(tailhook) wakeup remote coroutine
    }
    pub fn get_values(&mut self, f: &Filter) -> Vec<RemoteMetric> {
        let Filter { ref exact_key } = f;
        self.data.get(exact_key)
        .iter()
        .flat_map(|items| items.iter().map(|(h, v)| {
            RemoteMetric::from_local((exact_key, v).into(), h.clone())
        }))
        .collect()
    }
}
