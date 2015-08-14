use std::collections::{HashMap};

use super::Peers;

use query::{query_history, Dataset, Rule};


pub fn query(rules: &HashMap<String, Rule>, peers: &Peers)
    -> HashMap<String, HashMap<String, Dataset>>
{
    let mut items = HashMap::new();
    for (addr, &tok) in peers.addresses.iter() {
        let mut dict = HashMap::new();
        for (name, ref rule) in rules.iter() {
            dict.insert(name.clone(),
                query_history(rule, &peers.peers[tok].history));
        }
        items.insert(addr.to_string(), dict);
    }
    return items;
}

