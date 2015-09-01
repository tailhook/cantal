use std::collections::{HashMap};

use super::Peers;
use rustc_serialize::hex::ToHex;

use query::{query_history, Dataset, Rule};


pub fn query(rules: &HashMap<String, Rule>, peers: &Peers)
    -> HashMap<String, HashMap<String, Dataset>>
{
    let mut items = HashMap::new();
    for (id, &tok) in peers.tokens.iter() {
        let mut dict = HashMap::new();
        for (name, ref rule) in rules.iter() {
            dict.insert(name.clone(),
                query_history(rule, &peers.peers[tok].history));
        }
        items.insert(id.to_hex(), dict);
    }
    return items;
}

