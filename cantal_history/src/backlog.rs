use std::mem::replace;
use std::collections::{HashMap, VecDeque};
use num::{Float};

use values::Value as TipValue;
use super::deltabuf::{DeltaBuf, DeltaIter, Delta, Int};
use Key;

#[derive(Debug)]
pub struct Inner<T, U: ValueBuf<T>> {
    tip: T,
    age: u64,
    buf: U,
}

#[derive(Debug)]
pub enum Value {
    // value, age, delta-buffer
    Counter(Inner<u64, DeltaBuf<u64>>),
    Integer(Inner<i64, DeltaBuf<i64>>),
    Float(Inner<f64, VecDeque<f64>>),
}

impl Value {
    fn new(value: &TipValue, age: u64) -> Value {
        use self::Value as V;
        use values::Value as T;
        match value {
            &T::Counter(v) => V::Counter(Inner {
                tip: v,
                age: age,
                buf: DeltaBuf::new(),
            }),
            &T::Integer(v) => V::Integer(Inner {
                tip: v,
                age: age,
                buf: DeltaBuf::new(),
            }),
            &T::Float(v) => V::Float(Inner {
                tip: v,
                age: age,
                buf: VecDeque::new(),
            }),
            &T::State(_, _) => unreachable!(),
        }
    }
    fn push(&mut self, value: &TipValue, age: u64) -> bool {
        use self::Value as V;
        use values::Value as T;
        // If entry exists and it's type matches
        match (self, value) {
            (&mut V::Counter(ref mut b), &T::Counter(v)) => {
                b.push(v, age);
                return true;
            }
            (&mut V::Integer(ref mut b), &T::Integer(v)) => {
                b.push(v, age);
                return true;
            }
            (&mut V::Float(ref mut b), &T::Float(v)) => {
                b.push(v, age);
                return true;
            }
            _ => {}
        }
        return false;
    }
    fn truncate(&mut self, age: u64) -> bool {
        use self::Value as V;
        match self {
            &mut V::Counter(ref mut b) => b.truncate(age),
            &mut V::Integer(ref mut b) => b.truncate(age),
            &mut V::Float(ref mut b) => b.truncate(age),
        }
    }
}

#[derive(Debug)]
pub struct Backlog {
    age: u64,
    timestamps: VecDeque<(u64, u32)>,
    pub values: HashMap<Key, Value>,
}

#[derive(Clone, PartialEq, Eq, Copy, Debug)]
enum HState {
    Skip(u64),
    Tip,
    Diff,
}

#[derive(Clone)]
pub struct DeltaHistory<'a, T:Int> {
    state: HState,
    iter: DeltaIter<'a, T>,
    tip: T,
}

pub trait ValueBuf<T> {
    fn push(&mut self, old: T, new: T, age_diff: u64);
    fn truncate(&mut self, limit: usize);
}

impl<T: Copy, U:ValueBuf<T>> Inner<T, U> {
    fn push(&mut self, tip: T, age: u64) -> bool {
        if age < self.age {
            // Pushing already existing history
            // This condition should be true only on remote history being
            // pushed
            return false;
        }
        self.buf.push(self.tip, tip, age - self.age);
        self.tip = tip;
        self.age = age;
        return true;
    }
    /*
    fn history<'x>(&'x self, current_age: u64) -> ValueHistory<'x> {
        return DeltaHistory {
            state: if age_diff > 0 { Skip(age_diff) } else { Tip },
            iter: self.buf.deltas(),
            tip: self.tip,
        }
    }
    */
    fn truncate(&mut self, trim_age: u64) -> bool {
        if self.age <= trim_age {
            return false;
        }
        self.buf.truncate((self.age - trim_age) as usize);
        return true;
    }
    pub fn tip(&self) -> T {
        self.tip
    }
}

impl<'a, T:Int> Iterator for DeltaHistory<'a, T> {
    type Item = Option<T>;
    fn next(&mut self) -> Option<Option<T>> {
        use self::HState::*;
        let (res, nstate) = match self.state {
            Skip(1) => (Some(None), Tip),
            Skip(x) => (Some(None), Skip(x-1)),
            Tip => (Some(Some(self.tip)), Diff),
            Diff => {
                let res = match self.iter.next() {
                    Some(Delta::Positive(x)) => {
                        self.tip = self.tip - x;
                        Some(Some(self.tip))
                    }
                    Some(Delta::Negative(x)) => {
                        self.tip = self.tip + x;
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

impl<T:Int> ValueBuf<T> for DeltaBuf<T> {
    fn push(&mut self, old: T, new: T, age_diff: u64) {
        DeltaBuf::push(self, old, new, age_diff)
    }
    fn truncate(&mut self, limit: usize) {
        DeltaBuf::truncate(self, limit.saturating_sub(1));
    }
}

impl<T:Float> ValueBuf<T> for VecDeque<T> {
    fn push(&mut self, _old: T, new: T, age_diff: u64) {
        if age_diff > 1 {
            for _ in 1..age_diff {
                self.push_front(T::nan());
            }
        }
        self.push_front(new);
    }
    fn truncate(&mut self, limit: usize) {
        // TODO(tailhook) use truncate
        while self.len() > limit {
            self.pop_back();
        }
    }
}

impl Backlog {
    pub fn new() -> Backlog {
        Backlog {
            age: 0,
            timestamps: VecDeque::new(),
            values: HashMap::new(),
        }
    }
    pub fn push<'x, I>(&mut self, timestamp: (u64, u32), iter: I)
        where I: Iterator<Item=(&'x Key, &'x TipValue)>
    {
        assert!(self.timestamps.len() == 0 ||
                timestamp.0 > self.timestamps[0].0);
        self.timestamps.push_front(timestamp);
        self.age += 1;
        let age = self.age;
        for (k, v) in iter {
            // fast path should be get_mut
            if !self.values.get_mut(k).map(|x| x.push(v, age))
                .unwrap_or(false)
            {
                // Only if no key or conflicting type clone the key
                self.values.insert(k.clone(), Value::new(v, age));
            }
        }
    }
    pub fn truncate_by_time(&mut self, timestamp: u64) {
        if let Some((idx, _)) = self.timestamps.iter().enumerate()
            .find(|&(_idx, &(ts, _dur))| ts < timestamp)
        {
            self.truncate_by_num(idx);
        }
    }
    pub fn truncate_by_num(&mut self, idx: usize) {
        let target_age = self.age.saturating_sub(idx as u64);
        self.values = replace(&mut self.values, HashMap::new()).into_iter()
            .filter_map(|(key, mut val)| {
                if val.truncate(target_age) {
                    return Some((key, val));
                } else {
                    return None;
                }
            }).collect();
    }
}

#[cfg(test)]
mod test {
    use {Backlog, Key, ValueSet};
    use values::Value::Counter;
    use std::collections::{HashMap, HashSet};


    #[test]
    fn test_simple() {
        let mut backlog = Backlog::new();
        backlog.push((1000, 10), vec![
            (&Key::metric("test1"), &Counter(10)),
            (&Key::metric("test2"), &Counter(20)),
        ].iter());
        backlog.push((2000, 10), vec![
            (&Key::metric("test2"), &Counter(20)),
            (&Key::metric("test3"), &Counter(30)),
        ].iter());
        assert_eq!(backlog.age, 2);
        assert_eq!(backlog.values.len(), 3);
    }
}
