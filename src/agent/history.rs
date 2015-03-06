use std::f64::NAN;
use std::cmp::min;
use std::num::Int;
use std::collections::VecDeque;
use std::collections::HashMap;
use std::collections::hash_map::Entry::{Occupied, Vacant};

use super::stats::Key;
use super::scan::Tip;
use cantal::Value as TipValue;


#[derive(Clone, Debug, Encodable, Decodable)]
pub enum Value {
    Counter(u64, u64, VecDeque<u8>),
    Integer(i64, u64, VecDeque<u8>),
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
            => Value::Counter(val, age, VecDeque::new()),
            TipValue::Integer(val)
            => Value::Integer(val, age, VecDeque::new()),
            TipValue::Float(val)
            => Value::Float(val, age, VecDeque::new()),
            TipValue::State(ts, val)
            => Value::State((ts, val), age),
        }
    }
}

fn add_level_delta(old_value: i64, new_value: i64,
                    mut age_diff: u64, buf: &mut VecDeque<u8>)
{
    if age_diff == 0 {
        warn!("Duplicate write at same age"); // Shouldn't we panic?
        return;
    }
    const SIGN_BIT: u8 = 0b00100000;
    const SKIP_BIT: u8 = 0b01000000;
    const MAX_DIFF: u64 = 63;
    const FIRST_BYTE_MASK: i64 = 0x00011111;
    const FIRST_BYTE_SHIFT: usize = 5;
    const CONTINUATION_BIT: u8 = 0b10000000;
    const CONTINUATION_MASK: i64 = 0x01111111;
    const CONTINUATION_SHIFT: usize = 7;
    age_diff -= 1;
    while age_diff > 0 {
        let cd = min(age_diff, MAX_DIFF);
        buf.push_front(SKIP_BIT | cd as u8);
        age_diff -= cd;
    }
    let (mut delta, sign) = if old_value > new_value {
        (old_value - new_value, SIGN_BIT)
    } else {
        (new_value - old_value, 0)
    };
    buf.push_front(sign | (delta & FIRST_BYTE_MASK) as u8);
    delta >>= FIRST_BYTE_SHIFT;
    while delta > 0 {
        buf.push_front((delta & CONTINUATION_MASK) as u8 | CONTINUATION_BIT);
        delta >>= CONTINUATION_SHIFT;
    }
}

fn add_counter_delta(old_value: u64, new_value: u64,
                    mut age_diff: u64, buf: &mut VecDeque<u8>)
{
    if age_diff == 0 {
        warn!("Duplicate write at same age"); // Shouldn't we panic?
        return;
    }
    const RESET_BIT: u8 = 0b00100000;
    const SKIP_BIT: u8 = 0b01000000;
    const MAX_DIFF: u64 = 63;
    const FIRST_BYTE_MASK: u64 = 0x00011111;
    const FIRST_BYTE_SHIFT: usize = 5;
    const CONTINUATION_BIT: u8 = 0b10000000;
    const CONTINUATION_MASK: u64 = 0x01111111;
    const CONTINUATION_SHIFT: usize = 7;
    age_diff -= 1;
    while age_diff > 0 {
        let cd = min(age_diff, MAX_DIFF);
        buf.push_front(SKIP_BIT | cd as u8);
        age_diff -= cd;
    }
    //  When old_value is bigger, we think of it as of counter reset
    //  and write new value instead of delta
    let (mut delta, sign) = if old_value > new_value {
        (new_value, RESET_BIT)
    } else {
        (new_value - old_value, 0)
    };
    buf.push_front(sign | (delta & FIRST_BYTE_MASK) as u8);
    delta >>= FIRST_BYTE_SHIFT;
    while delta > 0 {
        buf.push_front((delta & CONTINUATION_MASK) as u8 | CONTINUATION_BIT);
        delta >>= CONTINUATION_SHIFT;
    }
}

impl Value {
    fn push(&mut self, tip: TipValue, age: u64) -> Option<TipValue> {
        match self {
            &mut Value::Counter(ref mut oval, ref mut oage, ref mut buf) => {
                if let TipValue::Counter(nval) = tip {
                    add_counter_delta(*oval, nval, age - *oage, buf);
                    *oval = nval;
                    *oage = age;
                    return None;
                }
            }
            &mut Value::Integer(ref mut oval, ref mut oage, ref mut buf) => {
                if let TipValue::Integer(nval) = tip {
                    add_level_delta(*oval, nval, age - *oage, buf);
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
}

pub trait RawAccess {
    fn as_f64(&self) -> Option<f64>;
    fn as_u64(&self) -> Option<u64>;
}

impl RawAccess for Value {
    fn as_f64(&self) -> Option<f64> {
        match *self {
            Value::Float(x, _, _) => Some(x),
            _ => None,
        }
    }
    fn as_u64(&self) -> Option<u64> {
        match *self {
            Value::Counter(x, _, _) => Some(x),
            _ => None,
        }
    }
}
impl RawAccess for Option<Value> {
    fn as_f64(&self) -> Option<f64> {
        self.as_ref().and_then(|x| x.as_f64())
    }
    fn as_u64(&self) -> Option<u64> {
        self.as_ref().and_then(|x| x.as_u64())
    }
}
impl<'a> RawAccess for Option<&'a Value> {
    fn as_f64(&self) -> Option<f64> {
        self.as_ref().and_then(|x| x.as_f64())
    }
    fn as_u64(&self) -> Option<u64> {
        self.as_ref().and_then(|x| x.as_u64())
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
    // TEMPORARY
    pub fn get(&self, key: &Key) -> Option<&Value> {
        self.fine.get(key)
        .or_else(|| self.coarse.get(key))
    }
}
