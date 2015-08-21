use values::Value;
use {Chunk, TimeStamp};


#[derive(Debug)]
pub enum ChunkSet<'a> {
    Empty,
    States(Vec<&'a (TimeStamp, String)>),
    Counters(Vec<&'a Vec<Option<u64>>>),
    Integers(Vec<&'a Vec<Option<i64>>>),
    Floats(Vec<&'a Vec<Option<f64>>>),
    Conflict,
}

#[derive(Debug)]
pub enum ValueSet<'a> {
    Empty,
    States(Vec<&'a (TimeStamp, String)>),
    Counters(Vec<u64>),
    Integers(Vec<i64>),
    Floats(Vec<f64>),
    Conflict,
}


impl<'a> ChunkSet<'a> {
    pub fn join(mut self, value: &'a Chunk) -> ChunkSet {
        use ChunkSet as S;
        use Chunk as C;
        match self {
            S::Empty => match value {
                &C::State(ref item) => S::States(vec![item]),
                &C::Counter(ref item) => S::Counters(vec![item]),
                &C::Integer(ref item) => S::Integers(vec![item]),
                &C::Float(ref item) => S::Floats(vec![item]),
            },
            S::Conflict => S::Conflict,
            _ => {
                match (&mut self, value) {
                    (&mut S::States(ref mut x), &C::State(ref item)) => {
                        x.push(item);
                    }
                    (&mut S::Counters(ref mut x), &C::Counter(ref item)) => {
                        x.push(item);
                    }
                    (&mut S::Integers(ref mut x), &C::Integer(ref item)) => {
                        x.push(item);
                    }
                    (&mut S::Floats(ref mut x), &C::Float(ref item)) => {
                        x.push(item);
                    }
                    _ => return S::Conflict,
                }
                self
            },
        }
    }
    pub fn merge<'x, I:Iterator<Item=&'x Chunk>>(iter: I) -> ChunkSet<'x> {
        iter.fold(ChunkSet::Empty, ChunkSet::join)
    }
}

impl<'a> ValueSet<'a> {
    pub fn join(mut self, value: &'a Value) -> ValueSet {
        use ValueSet as S;
        use values::Value as V;
        match self {
            S::Empty => match value {
                &V::State(ref item) => S::States(vec![item]),
                &V::Counter(item) => S::Counters(vec![item]),
                &V::Integer(item) => S::Integers(vec![item]),
                &V::Float(item) => S::Floats(vec![item]),
            },
            S::Conflict => S::Conflict,
            _ => {
                match (&mut self, value) {
                    (&mut S::States(ref mut x), &V::State(ref item)) => {
                        x.push(item);
                    }
                    (&mut S::Counters(ref mut x), &V::Counter(item)) => {
                        x.push(item);
                    }
                    (&mut S::Integers(ref mut x), &V::Integer(item)) => {
                        x.push(item);
                    }
                    (&mut S::Floats(ref mut x), &V::Float(item)) => {
                        x.push(item);
                    }
                    _ => return S::Conflict,
                }
                self
            },
        }
    }
    pub fn merge<'x, I:Iterator<Item=&'x Value>>(iter: I) -> ValueSet<'x> {
        iter.fold(ValueSet::Empty, ValueSet::join)
    }
}
