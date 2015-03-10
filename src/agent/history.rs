use std::f64::NAN;
use std::cmp::min;
use std::collections::VecDeque;
use std::collections::HashMap;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use serialize::json::Json;

use super::stats::Key;
use super::scan::Tip;
use super::deltabuf::{DeltaBuf, Delta};
use cantal::Value as TipValue;


#[derive(Debug, Encodable, Decodable)]
pub enum Value {
    Counter(u64, u64, DeltaBuf),
    Integer(i64, u64, DeltaBuf),
    Float(f64, u64, VecDeque<f64>),  // No compression, sorry
    State((u64, String), u64),  // No useful history
}

#[derive(Copy, Debug)]
pub enum Interval {
    Fine,       // very fine-grained (2-second)
    Coarse,     // coarse-grained (minute)
    Tip,        // No history
}


#[derive(Show, Encodable, Decodable)]
pub struct History {
    age: u64,

    // Values that are kept as fine-grained as possible (2-second interval)
    fine_timestamps: VecDeque<(u64, u32)>,
    fine: HashMap<Key, Value>,

    // Values that are kept at more coarse interval (1-minute)
    coarse_timestamps: VecDeque<(u64, u32)>,
    coarse: HashMap<Key, Value>,

    // Items that keep only last value
    tip_timestamp: (u64, u32),
    tip: HashMap<Key, TipValue>,
}


trait Source {
    fn default_interval(&self) -> Interval;
    fn begin_history(self, age: u64) -> Value;
}

impl Source for TipValue {
    fn default_interval(&self) -> Interval {
        match self {
            &TipValue::Counter(_) => Interval::Fine,
            &TipValue::Integer(_) => Interval::Fine,
            &TipValue::Float(_) => Interval::Coarse,
            &TipValue::State(_, _) => Interval::Tip,
        }
    }
    fn begin_history(self, age: u64) -> Value {
        match self {
            TipValue::Counter(val)
            => Value::Counter(val, age, DeltaBuf::new()),
            TipValue::Integer(val)
            => Value::Integer(val, age, DeltaBuf::new()),
            TipValue::Float(val)
            => Value::Float(val, age, VecDeque::new()),
            TipValue::State(ts, val)
            => Value::State((ts, val), age),
        }
    }
}

impl Value {
    fn push(&mut self, tip: TipValue, age: u64) -> Option<TipValue> {
        match self {
            &mut Value::Counter(ref mut oval, ref mut oage, ref mut buf) => {
                if let TipValue::Counter(nval) = tip {
                    buf.push(*oval, nval, age - *oage);
                    *oval = nval;
                    *oage = age;
                    return None;
                }
            }
            &mut Value::Integer(ref mut oval, ref mut oage, ref mut buf) => {
                if let TipValue::Integer(nval) = tip {
                    buf.push(*oval, nval, age - *oage);
                    *oval = nval;
                    *oage = age;
                    return None;
                }
            }
            &mut Value::Float(ref mut oval, ref mut oage, ref mut queue) => {
                if let TipValue::Float(nval) = tip {
                    for _ in *oage+1..age {
                        queue.push_front(NAN);
                    }
                    queue.push_front(nval);
                    *oage = age;
                    *oval = nval;
                    return None;
                }
            }
            _ => {},
        }
        return Some(tip);
    }
    fn json_history(&self, mut num: usize, current_age: u64) -> Json {
        match self {
            &Value::Counter(tip, age, ref buf) => {
                let mut res = vec!();
                for _ in 0..min(current_age - age, num as u64) {
                    num -= 1;
                    res.push(Json::Null);
                }
                res.push(Json::U64(tip));
                num -= 1;
                let mut val = tip;
                for dlt in buf.deltas(num) {
                    match dlt {
                        Delta::Positive(x) => {
                            val -= x as u64;
                            res.push(Json::U64(val));
                        }
                        Delta::Negative(x) => {
                            val += x as u64;
                            res.push(Json::U64(val));
                        }
                        Delta::Skip => res.push(Json::Null),
                    }
                }
                return Json::Array(res);
            }
            &Value::Integer(tip, age, ref buf) => {
                let mut res = vec!();
                for _ in 0..min(current_age - age, num as u64) {
                    num -= 1;
                    res.push(Json::Null);
                }
                res.push(Json::I64(tip));
                num -= 1;
                let mut val = tip;
                for dlt in buf.deltas(num) {
                    match dlt {
                        Delta::Positive(x) => {
                            val -= x as i64;
                            res.push(Json::I64(val));
                        }
                        Delta::Negative(x) => {
                            val += x as i64;
                            res.push(Json::I64(val));
                        }
                        Delta::Skip => res.push(Json::Null),
                    }
                }
                return Json::Array(res);
            }
            &Value::Float(tip, age, ref buf) => {
                let mut res = vec!();
                for _ in 0..min(current_age - age, num as u64) {
                    num -= 1;
                    res.push(Json::Null);
                }
                res.push(Json::F64(tip));
                num -= 1;
                for (idx, val) in buf.iter().enumerate() {
                    if idx > num { break; }
                    res.push(Json::F64(*val));
                }
                return Json::Array(res);
            }
            &Value::State((ts, ref text), age) => {
                return Json::Null;  // No history for State
            }
        }
    }
}

impl History {
    pub fn new() -> History {
        return History {
            age: 0,
            fine_timestamps: VecDeque::new(),
            fine: HashMap::new(),
            coarse_timestamps: VecDeque::new(),
            coarse: HashMap::new(),
            tip_timestamp: (0, 0),
            tip: HashMap::new(),
        }
    }
    pub fn push(&mut self, timestamp: u64, duration: u32, tip: Tip) {
        self.age += 1;
        for (key, value) in tip.map.into_iter() {
            let collection = match value.default_interval() {
                Interval::Fine => &mut self.fine,
                Interval::Coarse => &mut self.coarse,
                Interval::Tip => {
                    self.tip.insert(key, value);
                    continue;
                }
            };
            match collection.entry(key) {
                Occupied(mut entry) => {
                    if let Some(val) = entry.get_mut().push(value, self.age) {
                        *entry.get_mut() = val.begin_history(self.age);
                    }
                }
                Vacant(entry) => {
                    entry.insert(value.begin_history(self.age));
                }
            }
        }
        self.fine_timestamps.push_front((timestamp, duration));
        self.coarse_timestamps.push_front((timestamp, duration));
        self.tip_timestamp = (timestamp, duration);
    }
    pub fn get_tip_json(&self, key: &Key) -> Json {
        self.fine.get(key)
        .or_else(|| self.coarse.get(key))
        .map(|x| match *x {
            Value::Counter(c, _, _) => Json::U64(c),
            Value::Integer(c, _, _) => Json::I64(c),
            Value::Float(c, _, _) => Json::F64(c),
            Value::State((ts, ref text), _) => Json::Array(vec!(
                Json::U64(ts),
                Json::String(text.clone()),
                )),
        })
        .or_else(||
            self.tip.get(key)
            .map(|x| match *x {
                TipValue::Counter(c) => Json::U64(c),
                TipValue::Integer(c) => Json::I64(c),
                TipValue::Float(c) => Json::F64(c),
                TipValue::State(ts, ref text) => Json::Array(vec!(
                    Json::U64(ts),
                    Json::String(text.clone()),
                    )),
            }))
        .unwrap_or(Json::Null)
    }
    pub fn get_history_json(&self, key: &Key, num: usize) -> Json {
        self.fine.get(key)
            .map(|x| Json::Object(vec!(
                ("fine".to_string(), x.json_history(num, self.age)),
                ).into_iter().collect()))
        .or_else(|| self.coarse.get(key)
            .map(|x| Json::Object(vec!(
                ("coarse".to_string(), x.json_history(num, self.age)),
                ).into_iter().collect())))
        .unwrap_or(Json::Null)
    }
    pub fn get_timestamps(&self, num: usize) -> Vec<(u64, u32)> {
         self.fine_timestamps.iter().take(num).cloned().collect()
    }
    pub fn filter<'x, F:Fn(&Key) -> bool>(&'x self, predicate: F)
        -> Vec<(Json, Json)>
    {
        let mut res = Vec::new();
        for (key, _) in self.fine.iter() {
            if predicate(key) {
                // TODO(tailhook) optimize lookups
                res.push((key.to_json(), self.get_tip_json(key)));
            }
        }
        for (key, _) in self.coarse.iter() {
            if predicate(key) {
                // TODO(tailhook) optimize lookups
                res.push((key.to_json(), self.get_tip_json(key)));
            }
        }
        for (key, _) in self.tip.iter() {
            if predicate(key) {
                // TODO(tailhook) optimize lookups
                res.push((key.to_json(), self.get_tip_json(key)));
            }
        }
        return res;
    }
}
