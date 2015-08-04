use std::collections::HashMap;
use std::collections::BTreeMap;

use rustc_serialize::json::Json;
use rustc_serialize::json::ToJson;

use super::{Peers, Peer};
use super::super::rules::{Rule, Query, Source, json_tree, match_cond};
use super::super::rules::{take_rate, take_raw, sum_rate, sum_raw};
use history::ValueSet;



fn query_fine(rule: &Rule, peer: &Peer) -> Json {
    use super::super::rules::Aggregation::*;
    let mut keys = HashMap::<_, ValueSet>::new();
    for (ref key, value) in peer.history.fine.iter() {
        if match_cond(key, &rule.condition) {
            let target_key = rule.key.iter()
                             .map(|x| key.get(x).unwrap_or(""))
                             .collect::<Vec<_>>();
            keys.entry(target_key)
                .or_insert(ValueSet::Empty)
                .push(value);
        }
    }
    return json_tree(keys.into_iter().map(|(key, stream)| {
        use history::ValueSet::*;
        let ts = &peer.history.fine_timestamps;
        let json = match rule.aggregation {
            None => stream.map(|s| match s {
                Empty => Json::Null,
                Counters(x) => take_rate(ts, x, rule.limit),
                Integers(x) => take_raw(x, rule.limit),
                Floats(x) => take_raw(x, rule.limit),
            }),
            CasualSum => stream.map(|s| match s {
                Empty => Json::Null,
                Counters(x) => sum_rate(ts, x, rule.limit),
                Integers(x) => sum_raw(x, rule.limit, 0),
                Floats(x) => sum_raw(x, rule.limit, 0.),
            }),
        };
        (key, json.unwrap_or(Json::Null))
    }))
}


pub fn query(query: &Query, peers: &Peers) -> Json {
    // TODO(tailhook) query.limit is unused, remove it or use it
    let mut items = BTreeMap::new();
    for (addr, &tok) in peers.addresses.iter() {
        let mut dict = BTreeMap::new();
        for (name, ref rule) in query.rules.iter() {
            match rule.source {
                Source::Tip => unimplemented!(),
                Source::Fine => {
                    dict.insert(name.clone(),
                        query_fine(rule, &peers.peers[tok]));
                }
                Source::Coarse => unimplemented!(),
            }
        }
        items.insert(addr.to_string(), Json::Object(vec![
            ("fine_metrics".to_string(), Json::Object(dict)),
            ("fine_timestamps".to_string(),
                peers.peers[tok].history.fine_timestamps
                .iter().map(|&(x, _)| x).collect::<Vec<_>>()
                .to_json()),
            ].into_iter().collect()));
    }
    return Json::Object(items);
}

