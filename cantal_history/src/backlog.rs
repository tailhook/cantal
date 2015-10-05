use std::mem::{replace, size_of, size_of_val};
use std::collections::{HashMap, VecDeque};
use std::collections::vec_deque::Iter as VecDequeIter;

use num::{Float};
use serialize::json::{Json, ToJson};

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

probor_enum_encoder_decoder!(Value {
    #0 Counter(inner #1),
    #1 Integer(inner #1),
    #2 Float(inner #1),
});


#[derive(Debug)]
pub struct Backlog {
    // Made pub for serializer, may be fix it?
    pub age: u64,
    pub timestamps: VecDeque<(u64, u32)>,
    pub values: HashMap<Key, Value>,
}

// Named fields are ok since we don't store lots of History objects
probor_struct_encoder_decoder!(Backlog {
    age => (),
    timestamps => (),
    values => (),
});

#[derive(Clone, PartialEq, Eq, Copy, Debug)]
enum HState {
    Skip(u64),
    Tip,
    Next,
}

#[derive(Clone)]
pub struct DeltaHistory<'a, T:Int> {
    state: HState,
    iter: DeltaIter<'a, T>,
    tip: T,
}

#[derive(Clone)]
pub struct FloatHistory<'a, T:Float+Copy+'static> {
    state: HState,
    iter: VecDequeIter<'a, T>,
    tip: T,
}

impl Value {
    /// Size of key in bytes, for debugging
    pub fn size(&self) -> usize {
        match self {
            &Value::Counter(ref i) => size_of_val(self) + i.buf.size(),
            &Value::Integer(ref i) => size_of_val(self) + i.buf.size(),
            &Value::Float(ref i) => size_of_val(self) + i.buf.size(),
        }
    }
    pub fn new(value: &TipValue, age: u64) -> Value {
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
            &T::State(_) => unreachable!(),
        }
    }
    pub fn push(&mut self, value: &TipValue, age: u64) -> bool {
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
    pub fn age(&self) -> u64 {
        use self::Value::*;
        match self {
            &Counter(ref b) => b.age(),
            &Integer(ref b) => b.age(),
            &Float(ref b) => b.age(),
        }
    }
    pub fn tip_value(&self) -> TipValue {
        use self::Value as S;
        use values::Value as D;
        match self {
            &S::Counter(ref b) => D::Counter(b.tip()),
            &S::Integer(ref b) => D::Integer(b.tip()),
            &S::Float(ref b) => D::Float(b.tip()),
        }
    }
}

pub trait ValueBuf<T> {
    fn push(&mut self, old: T, new: T, age_diff: u64);
    fn truncate(&mut self, limit: usize);
    fn size(&self) -> usize;
}

impl<T: Copy, U:ValueBuf<T>> Inner<T, U> {
    pub fn unpack<S:Into<U>>(tip: T, age: u64, buf: S) -> Inner<T, U> {
        Inner {
            tip: tip,
            age: age,
            buf: buf.into(),
        }
    }
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
    pub fn age(&self) -> u64 {
        self.age
    }
    pub fn buf<'x>(&'x self) -> &'x U {
        &self.buf
    }
}

impl<'a, T:Int> Iterator for DeltaHistory<'a, T> {
    type Item = Option<T>;
    fn next(&mut self) -> Option<Option<T>> {
        use self::HState::*;
        let (res, nstate) = match self.state {
            Skip(1) => (Some(None), Tip),
            Skip(x) => (Some(None), Skip(x-1)),
            Tip => (Some(Some(self.tip)), Next),
            Next => {
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
                (res, Next)
            }
        };
        self.state = nstate;
        return res;
    }
}

impl<'a, T:Float> Iterator for FloatHistory<'a, T> {
    type Item = Option<T>;
    fn next(&mut self) -> Option<Option<T>> {
        use self::HState::*;
        let (res, nstate) = match self.state {
            Skip(1) => (Some(None), Tip),
            Skip(x) => (Some(None), Skip(x-1)),
            Tip => (Some(Some(self.tip)), Next),
            Next => {
                let val = self.iter.next()
                    .map(|x| if x.is_nan() { None } else { Some(*x) });
                (val, Next)
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
    fn size(&self) -> usize {
        self.byte_size()
    }
}

impl <T:Int> Inner<T, DeltaBuf<T>> {
    pub fn history<'x>(&'x self, current_age: u64) -> DeltaHistory<'x, T> {
        use self::HState::*;
        let age_diff = current_age.saturating_sub(self.age());
        return DeltaHistory {
            state: if age_diff > 0 { Skip(age_diff) } else { Tip },
            iter: self.buf.deltas(),
            tip: self.tip,
        }
    }
}


impl<T: Float> Inner<T, VecDeque<T>> {
    pub fn history<'x>(&'x self, current_age: u64) -> FloatHistory<'x, T> {
        use self::HState::*;
        let age_diff = current_age.saturating_sub(self.age());
        return FloatHistory {
            state: if age_diff > 0 { Skip(age_diff) } else { Tip },
            iter: self.buf.iter(),
            tip: self.tip,
        }
    }
}

impl<T:Float+Sized> ValueBuf<T> for VecDeque<T> {
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
    fn size(&self) -> usize {
        self.len()*size_of::<T>()
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
    pub fn info(&self) -> Json {
        let mut key_bytes = 0;
        let mut value_bytes = 0;
        for (k, v) in self.values.iter() {
            key_bytes += k.size();
            value_bytes += v.size();
        }
        return Json::Object(vec![
            ("age".to_string(), self.age.to_json()),
            ("timestamps".to_string(), self.timestamps.len().to_json()),
            ("values".to_string(), self.values.len().to_json()),
            ("key_bytes".to_string(), key_bytes.to_json()),
            ("value_bytes".to_string(), value_bytes.to_json()),
            ].into_iter().collect());
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

mod serde {
    use std::io::Cursor;
    use std::collections::VecDeque;
    use probor::{Decodable, Decoder, DecodeError, Input};
    use probor::{Encodable, Encoder, EncodeError, Output};
    use byteorder::{WriteBytesExt, BigEndian, ReadBytesExt};
    use cbor::types::Type;
    use super::Inner;
    use super::super::deltabuf::{DeltaBuf, Int};

    fn type_len<W:Output>(w: &mut W, t: Type, x: u64) {
        match x {
            0...23
            => w.write_u8(t.major() << 5 | x as u8).unwrap(),
            24...0xFF
            => w.write_u8(t.major() << 5 | 24).and(w.write_u8(x as u8)).unwrap(),
            0x100...0xFFFF
            => w.write_u8(t.major() << 5 | 25)
               .and(w.write_u16::<BigEndian>(x as u16)).unwrap(),
            0x100000...0xFFFFFFFF
            => w.write_u8(t.major() << 5 | 26)
               .and(w.write_u32::<BigEndian>(x as u32)).unwrap(),
            _
            => w.write_u8(t.major() << 5 | 27)
               .and(w.write_u64::<BigEndian>(x)).unwrap(),
        }
    }


    fn write_bytes<W:Output, F>(e: &mut Encoder<W>, num: usize, mut fun: F)
        where F: FnMut(&mut W)
    {
        let mut v = e.writer();
        type_len(v, Type::Bytes, num as u64);
        fun(&mut v);
    }
    struct Bytes(Vec<u8>);
    impl Decodable for Bytes {
        fn decode_opt<R:Input>(d: &mut Decoder<R>)
            -> Result<Option<Self>, DecodeError>
        {
            d.bytes().map(Bytes).map(Some)
            .map_err(|e| DecodeError::WrongType("expected bytes", e))
        }
    }

    impl<T:Decodable+Int> Decodable for Inner<T, DeltaBuf<T>> {
        fn decode_opt<R:Input>(d: &mut Decoder<R>)
            -> Result<Option<Self>, DecodeError>
        {
            probor_dec_struct!(d, {
                tip => (#0),
                age => (#1),
                buf => (#2),
            });
            let Bytes(buf) = buf;
            Ok(Some(Inner::unpack(tip, age, buf)))
        }
    }

    impl<T:Encodable+Int> Encodable for Inner<T, DeltaBuf<T>> {
        fn encode<W:Output>(&self, e: &mut Encoder<W>)
            -> Result<(), EncodeError>
        {
            try!(e.array(3));  // {tip, age, buf}
            try!(self.tip().encode(e));  // #0
            try!(self.age().encode(e));  // #1
            write_bytes(e, self.buf().bytes().len(), |buf| {  // #2
                // I hope this crap will be optimized
                for &i in self.buf().bytes() {
                    buf.write_all(&[i]).unwrap()
                }
            });
            Ok(())
        }
    }


    impl Decodable for Inner<f64, VecDeque<f64>> {
        fn decode_opt<R:Input>(d: &mut Decoder<R>)
            -> Result<Option<Self>, DecodeError>
        {
            probor_dec_struct!(d, {
                tip => (#0),
                age => (#1),
                buf => (#2),
            });
            let mut deque = VecDeque::new();
            let Bytes(buf) = buf;
            if buf.len() % 8 > 0 {
                return Err(DecodeError::WrongValue("length of f64 buffer \
                    should be multiple of 8"));
            }
            let num = buf.len() / 8;
            let mut cur = Cursor::new(buf);
            for _ in 0..num {
                deque.push_back(
                    cur.read_f64::<BigEndian>().unwrap());
            }
            Ok(Some(Inner::unpack(tip, age, deque)))
        }
    }

    impl Encodable for Inner<f64, VecDeque<f64>> {
        fn encode<W:Output>(&self, e: &mut Encoder<W>)
            -> Result<(), EncodeError>
        {
            try!(e.array(3));  // {tip, age, buf}
            try!(self.tip().encode(e));  // #0
            try!(self.age().encode(e));  // #1
            write_bytes(e, self.buf().len()*8, |buf| {  // #2
                for val in self.buf().iter() {
                    buf.write_f64::<BigEndian>(*val).unwrap();
                }
            });
            Ok(())
        }
    }

}

#[cfg(test)]
mod test {
    use std::io::Cursor;
    use {Backlog, Key};
    use super::{Value, Inner};
    use values::Value::Counter;
    use std::collections::{HashMap, HashSet};
    use probor::{Encodable, Decodable, Encoder, Decoder, Config, decode};


    #[test]
    fn test_simple() {
        let mut backlog = Backlog::new();
        backlog.push((1000, 10), vec![
            (&Key::metric("test1"), &Counter(10)),
            (&Key::metric("test2"), &Counter(20)),
        ].into_iter());
        backlog.push((2000, 10), vec![
            (&Key::metric("test2"), &Counter(20)),
            (&Key::metric("test3"), &Counter(30)),
        ].into_iter());
        assert_eq!(backlog.age, 2);
        assert_eq!(backlog.values.len(), 3);
    }

    fn roundtrip<T:Encodable+Decodable>(v: &T) -> T {
        let mut e = Encoder::new(Vec::new());
        v.encode(&mut e).unwrap();
        let val = &e.into_writer()[..];
        println!("Serialized {:?}", val);
        decode(&mut Decoder::new(Config::default(), Cursor::new(val))).unwrap()
    }

    #[test]
    fn test_serde() {
        let mut value = Value::Counter(Inner::unpack(10, 1, vec![]));
        value.push(&Counter(20), 2);
        let nval:Value = roundtrip(&value);
        if let Value::Counter(inner) = nval {
            assert_eq!(inner.tip(), 20);
            assert_eq!(inner.age(), 2);
        }
    }
}
