use std::collections::VecDeque;

use super::backlog::{Inner, Value};
use super::deltabuf::DeltaBuf;


#[derive(Debug)]
pub enum ValueSet<'a> {
    Empty,
    Counters(Vec<&'a Inner<u64, DeltaBuf<u64>>>),
    Integers(Vec<&'a Inner<i64, DeltaBuf<i64>>>),
    Floats(Vec<&'a Inner<f64, VecDeque<f64>>>),
    Error,
}


impl<'a> ValueSet<'a> {
    pub fn join(&mut self, value: &'a Value) {
        use ValueSet as S;
        use super::backlog::Value as H;
        if let &mut S::Empty = self {
            *self = match value {
                &H::Counter(ref item) => S::Counters(vec![item]),
                &H::Integer(ref item) => S::Integers(vec![item]),
                &H::Float(ref item) => S::Floats(vec![item]),
            };
        } else if let &mut S::Error = self {
            // Nothing to do
            return;
        }
        let mut me = self;
        match (&mut me, value) {
            (&mut &mut S::Counters(ref mut x), &H::Counter(ref item)) => {
                x.push(item);
                return;
            }
            (&mut &mut S::Integers(ref mut x), &H::Integer(ref item)) => {
                x.push(item);
                return;
            }
            (&mut &mut S::Floats(ref mut x), &H::Float(ref item)) => {
                x.push(item);
                return;
            }
            _ => {}  // Any other combinations -> error
        }
        *me = S::Error;
    }
}
