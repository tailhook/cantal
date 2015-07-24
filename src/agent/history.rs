use std::f64::NAN;
use std::cmp::min;
use std::mem::replace;
use std::iter::{repeat, Take, Repeat, Chain};
use std::collections::VecDeque;
use std::collections::vec_deque::Iter as DequeIter;
use std::collections::HashMap;
use std::collections::hash_map::Entry::{Occupied, Vacant};

use super::stats::Key;
use super::scan::Tip;
use super::rules::RawResult;
use super::deltabuf::{DeltaBuf, DeltaIter, Delta};
use super::history_chunk::HistoryChunk;
use cantal::Value as TipValue;

type FloatHistory<'a> = Chain<Take<Repeat<Option<f64>>>, NoneMap<'a>>;

#[derive(Debug, RustcEncodable, RustcDecodable)]
pub enum Value {
    // value, age, delta-buffer
    Counter(u64, u64, DeltaBuf),
    Integer(i64, u64, DeltaBuf),
    Float(f64, u64, VecDeque<f64>),  // No compression, sorry
    // (timestamp, text), age
    State((u64, String), u64),  // No useful history
}

pub enum ValueHistory<'a> {
    Counter(CounterHistory<'a>),
    Integer(IntegerHistory<'a>),
    Float(FloatHistory<'a>),
}


pub enum Histories<'a> {
    Empty,
    Counters(Vec<CounterHistory<'a>>),
    Integers(Vec<IntegerHistory<'a>>),
    Floats(Vec<FloatHistory<'a>>),
}

#[derive(Clone, Copy, Debug)]
pub enum Interval {
    Fine,       // very fine-grained (2-second)
    Coarse,     // coarse-grained (minute)
    Tip,        // No history
}

#[derive(Debug, RustcEncodable, RustcDecodable)]
pub struct History {
    pub age: u64,

    // Values that are kept as fine-grained as possible (2-second interval)
    pub fine_timestamps: VecDeque<(u64, u32)>,
    pub fine: HashMap<Key, Value>,

    // Values that are kept at more coarse interval (1-minute)
    pub coarse_timestamps: VecDeque<(u64, u32)>,
    pub coarse: HashMap<Key, Value>,

    // Items that keep only last value
    pub tip_timestamp: (u64, u32),
    pub tip: HashMap<Key, Value>,
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
            => Value::Float(val, age, vec![val].into_iter().collect()),
            TipValue::State(ts, val)
            => Value::State((ts, val), age),
        }
    }
}

impl<'a> ValueHistory<'a> {
    pub fn take(self, n: usize) -> HistoryChunk {
        use self::ValueHistory as S;
        use super::history_chunk::HistoryChunk as D;
        match self {
            S::Counter(x) => D::Counter(x.take(n).collect()),
            S::Integer(x) => D::Integer(x.take(n).collect()),
            S::Float(x) => D::Float(x.take(n).collect()),
        }
    }
}



#[derive(Clone)]
struct NoneMap<'a> {
    iter: DequeIter<'a, f64>,
}

impl<'a> Iterator for NoneMap<'a> {
    type Item = Option<f64>;
    fn next(&mut self) -> Option<Option<f64>> {
        self.iter.next().map(|x| if x.is_nan() { None } else { Some(*x) })
    }
}

impl Value {
    fn push(&mut self, tip: TipValue, age: u64) -> Result<bool, TipValue> {
        match self {
            &mut Value::Counter(ref mut oval, ref mut oage, ref mut buf) => {
                if let TipValue::Counter(nval) = tip {
                    if age <= *oage {
                        return Ok(false);
                    }
                    // In case deltabuf fails
                    // let mut deltas: Vec<_> = buf.deltas().collect();
                    // deltas.insert(0, Delta::Positive(nval - *oval));
                    buf.push(*oval as i64, nval as i64, age - *oage);
                    // assert_eq!(buf.deltas().collect::<Vec<_>>(), deltas);
                    *oval = nval;
                    *oage = age;
                    return Ok(true);
                }
            }
            &mut Value::Integer(ref mut oval, ref mut oage, ref mut buf) => {
                if let TipValue::Integer(nval) = tip {
                    if age <= *oage {
                        return Ok(false);
                    }
                    buf.push(*oval, nval, age - *oage);
                    *oval = nval;
                    *oage = age;
                    return Ok(true);
                }
            }
            &mut Value::Float(ref mut oval, ref mut oage, ref mut queue) => {
                if let TipValue::Float(nval) = tip {
                    if age <= *oage {
                        return Ok(false);
                    }
                    for _ in *oage+1..age {
                        queue.push_front(NAN);
                    }
                    queue.push_front(nval);
                    *oage = age;
                    *oval = nval;
                    return Ok(true);
                }
            }
            _ => {},
        }
        return Err(tip);
    }
    fn history<'x>(&'x self, current_age: u64) -> ValueHistory<'x> {
        use self::ValueHistory::*;
        match self {
            &Value::Counter(tip, age, ref buf)
            => Counter(CounterHistory::new(tip, current_age - age, buf)),
            &Value::Integer(tip, age, ref buf)
            => Integer(IntegerHistory::new(tip, current_age - age, buf)),
            &Value::Float(_, age, ref buf)
            => Float(repeat(None).take((current_age - age) as usize)
                     .chain(NoneMap { iter: buf.iter() })),
            &Value::State(_, _)
            => unreachable!(),
        }
    }
    fn truncate(&mut self, trim_age: u64) -> bool {
        match self {
            &mut Value::Counter(_, age, ref mut buf) => {
                if age <= trim_age {
                    return false;
                } else {
                    buf.truncate((age - trim_age) as usize);
                }
            }
            &mut Value::Integer(_, age, ref mut buf) => {
                if age <= trim_age {
                    return false;
                } else {
                    buf.truncate((age - trim_age) as usize);
                }
            }
            &mut Value::Float(_, age, ref mut queue) => {
                if age <= trim_age {
                    return false;
                } else {
                    // TODO(tailhook) fixme: use truncate
                    while queue.len() > (age - trim_age) as usize {
                        queue.pop_back();
                    }
                    //queue.truncate((age - trim_age) as usize);
                }
            }
            &mut Value::State(_, age) => {
                if age <= trim_age {
                    return false;
                }
            },
        }
        return true;
    }
}

#[derive(Clone, PartialEq, Eq, Copy, Debug)]
enum HState {
    Skip(u64),
    Tip,
    Diff,
}

#[derive(Clone)]
struct CounterHistory<'a> {
    state: HState,
    iter: DeltaIter<'a>,
    tip: u64,
}

impl<'a> CounterHistory<'a> {
    fn new<'x>(tip: u64, age_diff: u64, dbuf: &'x DeltaBuf)
        -> CounterHistory<'x>
    {
        use self::HState::*;
        CounterHistory {
            state: if age_diff > 0 { Skip(age_diff) } else { Tip },
            iter: dbuf.deltas(),
            tip: tip,
        }
    }
}

impl<'a> Iterator for CounterHistory<'a> {
    type Item = Option<u64>;
    fn next(&mut self) -> Option<Option<u64>> {
        use self::HState::*;
        let (res, nstate) = match self.state {
            Skip(1) => (Some(None), Tip),
            Skip(x) => (Some(None), Skip(x-1)),
            Tip => (Some(Some(self.tip)), Diff),
            Diff => {
                let res = match self.iter.next() {
                    Some(Delta::Positive(x)) => {
                        self.tip -= x as u64;
                        Some(Some(self.tip))
                    }
                    Some(Delta::Negative(x)) => {
                        self.tip += x as u64;
                        // Probably counter reset
                        Some(None)
                    }
                    Some(Delta::Skip) => Some(None),
                    None => None
                };
                (res, Diff)
            }
        };
        self.state = nstate;
        return res;
    }
}

#[derive(Clone)]
struct IntegerHistory<'a> {
    state: HState,
    iter: DeltaIter<'a>,
    tip: i64,
}

impl<'a> IntegerHistory<'a> {
    fn new<'x>(tip: i64, age_diff: u64, dbuf: &'x DeltaBuf)
        -> IntegerHistory<'x>
    {
        use self::HState::*;
        IntegerHistory {
            state: if age_diff > 0 { Skip(age_diff) } else { Tip },
            iter: dbuf.deltas(),
            tip: tip,
        }
    }
}

impl<'a> Iterator for IntegerHistory<'a> {
    type Item = Option<i64>;
    fn next(&mut self) -> Option<Option<i64>> {
        use self::HState::*;
        let (res, nstate) = match self.state {
            Skip(1) => (Some(None), Tip),
            Skip(x) => (Some(None), Skip(x-1)),
            Tip => (Some(Some(self.tip)), Diff),
            Diff => {
                let res = match self.iter.next() {
                    Some(Delta::Positive(x)) => {
                        self.tip -= x as i64;
                        Some(Some(self.tip))
                    }
                    Some(Delta::Negative(x)) => {
                        self.tip += x as i64;
                        Some(Some(self.tip))
                    }
                    Some(Delta::Skip) => Some(None),
                    None => None
                };
                (res, Diff)
            }
        };
        self.state = nstate;
        return res;
    }
}

/// Returns tuple of
/// ("number of new datapoints", "number of valid data points")
fn compare_timestamps(new: &Vec<(u64, u32)>, old: &VecDeque<(u64, u32)>)
    -> (u64, usize)
{
    let mut iter_new = new.iter().enumerate().peekable();
    let last_ots = old[0].0;
    let mut new_pt;
    loop { // New points
        match iter_new.peek() {
            None => return (new.len() as u64, new.len()),
            Some(&(_, &(nts, _))) if nts > last_ots => {
                iter_new.next().unwrap();
                continue;
            }
            Some(&(nidx, _)) => {
                new_pt = nidx;
                break;
            }
        }
    }
    for ((nidx, &(nts, _)), &(ots, _)) in iter_new.zip(old.iter()) {
        if nts != ots {
            return (new_pt as u64, nidx);
        }
    }
    return (new_pt as u64, min(new.len(), new_pt + old.len()));
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
    pub fn get_fine_history(&self, key: &Key) -> Option<ValueHistory> {
        self.fine.get(key).map(|v| v.history(self.age))
    }
    pub fn push(&mut self, timestamp: u64, duration: u32, tip: Tip) {
        self.age += 1;
        for (key, value) in tip.map.into_iter() {
            let collection = match value.default_interval() {
                Interval::Fine => &mut self.fine,
                Interval::Coarse => &mut self.coarse,
                Interval::Tip => {
                    match value {
                        TipValue::State(ts, val) => {
                            self.tip.insert(key,
                                Value::State((ts, val), self.age));
                        }
                        _ => unreachable!(),
                    }
                    continue;
                }
            };
            match collection.entry(key) {
                Occupied(mut entry) => {
                    if let Err(val) = entry.get_mut().push(value, self.age) {
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
    /// Checks and inserts timestamps which are not yet present in history
    /// Returns how many *valid* timestamps are there (usually all, but...)
    fn insert_timestamps(&mut self, timestamps: &Vec<(u64, u32)>) -> usize {
        if self.fine_timestamps.len() == 0 {
            self.fine_timestamps = timestamps.iter().cloned().collect();
            self.age += self.fine_timestamps.len() as u64;
            return self.fine_timestamps.len();
        } else {
            let (diff, valid) = compare_timestamps(
                timestamps, &self.fine_timestamps);
            self.age += diff;
            for &pair in timestamps[..diff as usize].iter().rev() {
                self.fine_timestamps.push_front(pair);
            }
            return valid;
        }
    }
    pub fn update_chunks(&mut self, chunk: RawResult) {
        if chunk.fine_timestamps.len() == 0 {
            debug!("Got empty timestamps {:?}", chunk);
            return
        }
        let valid = self.insert_timestamps(&chunk.fine_timestamps);
        for (key, data) in chunk.fine_metrics.into_iter() {
            let mut iter = data.iter().enumerate().rev();
            if let Some((foff, mut fval)) = iter.by_ref()
                .find(|&(off, ref val)| off < valid && val.is_some())
            {
                let foff = foff as u64;
                let age = self.age;
                let hist = self.fine.entry(Key::new(key))
                    .or_insert_with(|| {
                        fval.take().unwrap().begin_history(age - foff)
                    });
                if fval.is_some() {
                    if let Err(val) = hist.push(fval.take().unwrap(), age - foff) {
                        *hist = val.begin_history(age - foff);
                    }
                };
                for (off, val_opt) in iter {
                    let off = off as u64;
                    if let Some(val) = val_opt {
                        hist.push(val, age - off).unwrap();
                    }
                }
            }
        }
    }
    pub fn truncate_by_time(&mut self, timestamp: u64) {
        let idx = self.fine_timestamps.iter().enumerate()
            .skip_while(|&(_idx, &(ts, _dur))| ts >= timestamp)
            .next().unwrap_or((1000000, &(0, 0)))
            .0;

        let target_age = self.age.saturating_sub(idx as u64);
        self.fine = replace(&mut self.fine, HashMap::new()).into_iter()
            .filter_map(|(key, mut val)| {
                if val.truncate(target_age) {
                    return Some((key, val));
                } else {
                    return None;
                }
            }).collect();
        self.tip = replace(&mut self.tip, HashMap::new()).into_iter()
            .filter_map(|(key, mut val)| {
                if val.truncate(target_age) {
                    return Some((key, val));
                } else {
                    return None;
                }
            }).collect();
        // TODO(tailhook) fix to truncate
        while self.fine_timestamps.len() > idx+1 {
            self.fine_timestamps.pop_back();
        }

        let idx = self.coarse_timestamps.iter().enumerate()
            .skip_while(|&(_, &(ts, _))| ts >= timestamp)
            .next().unwrap_or((1000000, &(0, 0)))
            .0;

        let target_age = self.age.saturating_sub(idx as u64);
        self.coarse = replace(&mut self.coarse, HashMap::new()).into_iter()
            .filter_map(|(key, mut val)| {
                if val.truncate(target_age) {
                    return Some((key, val));
                } else {
                    return None;
                }
            }).collect();
        // TODO(tailhook) fix to truncate
        while self.coarse_timestamps.len() > idx+1 {
            self.coarse_timestamps.pop_back();
        }
    }
}

pub fn merge<'x, I: Iterator<Item=ValueHistory<'x>>>(iter: I)
    -> Option<Histories<'x>>
{
    use self::Histories::*;
    use self::ValueHistory::*;
    iter.fold(Some(Empty), |acc, val| acc.and_then(|a| Some(match (a, val) {
        (Empty, Counter(i)) => Counters(vec![i]),
        (Empty, Integer(i)) => Integers(vec![i]),
        (Empty, Float(i)) => Floats(vec![i]),
        (Counters(mut v), Counter(i)) => { v.push(i); Counters(v) }
        (Integers(mut v), Integer(i)) => { v.push(i); Integers(v) }
        (Floats(mut v), Float(i)) => { v.push(i); Floats(v) }
        // We don't use (_, _) below, to keep track of added pairs
        (Counters(_), _) => return None,
        (Integers(_), _) => return None,
        (Floats(_), _) => return None,
    })))
}

#[cfg(test)]
mod test {
    use super::compare_timestamps;

    #[test]
    fn all_new() {
        assert_eq!(compare_timestamps(
            &vec![(130, 0), (120, 0), (110, 0)],
            &vec![(30, 0), (20, 0), (10, 0)].into_iter().collect()),
            (3, 3));
    }

    #[test]
    fn touch() {
        assert_eq!(compare_timestamps(
            &vec![(50, 0), (40, 0), (30, 0)],
            &vec![(30, 0), (20, 0), (10, 0), (0, 0)].into_iter().collect()),
            (2, 3));
    }
    #[test]
    fn overlap() {
        assert_eq!(compare_timestamps(
            &vec![(40, 0), (30, 0), (20, 0)],
            &vec![(30, 0), (20, 0), (10, 0), (0, 0)].into_iter().collect()),
            (1, 3));
    }

    #[test]
    fn old() {
        assert_eq!(compare_timestamps(
            &vec![(30, 0), (20, 0), (10, 0)],
            &vec![(130, 0), (120, 0), (110, 0)].into_iter().collect()),
            (0, 0));
    }

    #[test]
    fn middle() {
        assert_eq!(compare_timestamps(
            &vec![(40, 0), (30, 0), (25, 0)],
            &vec![(30, 0), (20, 0), (10, 0), (0, 0)].into_iter().collect()),
            (1, 2));
    }

    #[test]
    fn middle2() {
        assert_eq!(compare_timestamps(
            &vec![(40, 0), (35, 0), (25, 0)],
            &vec![(30, 0), (20, 0), (10, 0), (0, 0)].into_iter().collect()),
            (2, 2));
    }

    #[test]
    fn middle3() {
        assert_eq!(compare_timestamps(
            &vec![(50, 0), (40, 0), (35, 0), (20, 0)],
            &vec![(30, 0), (20, 0), (10, 0), (0, 0)].into_iter().collect()),
            (3, 3));
    }

    #[test]
    fn new_big() {
        assert_eq!(compare_timestamps(
            &vec![(50, 0), (40, 0), (30, 0), (20, 0), (10, 0)],
            &vec![(30, 0), (20, 0)].into_iter().collect()),
            (2, 4));
    }
}

