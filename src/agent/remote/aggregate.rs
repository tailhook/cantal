use std::collections::{HashMap, BTreeMap};

use super::Peers;

use query::{query_history, Dataset, Rule};


fn query(rules: &HashMap<String, Rule>, peers: &Peers)
    -> HashMap<String, HashMap<String, Dataset>>
{
    let mut items = BTreeMap::new();
    for (addr, &tok) in peers.addresses.iter() {
        let mut dict = BTreeMap::new();
        for (name, ref rule) in rules.iter() {
            dict.insert(name.clone(),
                query_history(rule, &peers.peers[tok]));
        }
        items.insert(addr.to_string(), dict);
    }
    return items;
}

