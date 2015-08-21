use std::ops::Sub;

use num::traits::ToPrimitive;

use history::{Chunk, TimeStamp};
use {Dataset, Conflict};


pub fn non_negative_derivative(src: Dataset) -> Dataset {
    use Dataset::*;
    match src {
        MultiSeries(vec) => MultiSeries(vec.into_iter()
            .map(|(key, v, ts)| {
                let (nval, nts) = derive_series(v, ts);
                (key, nval, nts)
            }).collect()),
        SingleSeries(key, v, ts) => {
            let (nval, nts) = derive_series(v, ts);
            SingleSeries(key, nval, nts)
        },
        SingleTip(_, _, _) => Incompatible(Conflict::CantDerive),
        MultiTip(_) => Incompatible(Conflict::CantDerive),
        Chart(_) => Incompatible(Conflict::CantDerive),
        Incompatible(x) => Incompatible(x),
        Empty => Empty,
    }
}

fn derive_vec<T:Sub<T, Output=T>+Copy+ToPrimitive>(
    vec: Vec<Option<T>>, timestamps: Vec<TimeStamp>)
    -> (Chunk, Vec<TimeStamp>)
{
    let first = vec.iter().zip(&timestamps)
        .filter_map(|(v, t)| v.map(|x| (x, t)));
    let second = vec.iter().zip(&timestamps)
        .filter_map(|(v, t)| v.map(|x| (x, t))).skip(1);
    let (nval, ts) = first.zip(second).map(|((a, ta), (b, tb))| {
        (Some((a - b).to_f64().unwrap() * 1000. / (ta - tb) as f64), ta)
    }).unzip();
    (Chunk::Float(nval), ts)
}

fn derive_series(chunk: Chunk, timestamps: Vec<TimeStamp>)
    -> (Chunk, Vec<TimeStamp>)
{
    use history::Chunk::*;
    match chunk {
        State(x) => (State(x), timestamps),  // Should be Incompatible?
        Counter(items) => derive_vec(items, timestamps),
        Integer(items) => derive_vec(items, timestamps),
        Float(items) => derive_vec(items, timestamps),
    }
}
