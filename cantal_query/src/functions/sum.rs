use std::collections::HashMap;
use std::ops::Add;

use history::{Key, Chunk, ChunkSet, ValueSet, TimeStamp};
use values::Value;
use {Dataset, Conflict, TimeSlice};


pub fn sum(src: Dataset) -> Dataset {
    use Dataset::*;
    match src {
        MultiSeries(mut vec) => {
            if vec.len() == 1 {
                let (k, v, t) = vec.pop().unwrap();
                SingleSeries(k, v, t)
            } else {
                match sum_series(&vec) {
                    Ok((k, v, t)) => SingleSeries(k, v, t),
                    Err(c) => Incompatible(c),
                }
            }
        }
        MultiTip(vec) => sum_tip(vec),
        src @ SingleSeries(_, _, _) => src,
        src @ SingleTip(_, _, _) => src,
        src @ Incompatible(_) => src,
        Chart(_) => Incompatible(Conflict::CantSumChart),
        Empty => Empty,
    }
}

pub fn sum_by(by: &str, total: bool, src: Dataset) -> Dataset {
    use Dataset::*;
    match src {
        MultiSeries(vec) => match sum_series_by(by, vec) {
            Ok(mut vec) => {
                if total && vec.len() > 1 {
                    match sum_series(&vec) {
                        Ok(tup) => { vec.push(tup); }
                        Err(e) => return Incompatible(e),
                    }
                }
                MultiSeries(vec)
            }
            Err(c) => Incompatible(c),
        },
        MultiTip(vec) => unimplemented!(),
        src @ SingleSeries(_, _, _) => src,
        src @ SingleTip(_, _, _) => src,
        src @ Incompatible(_) => src,
        Chart(_) => Incompatible(Conflict::CantSumChart),
        Empty => Empty,
    }
}

fn sum_series_by(by: &str, vec: Vec<(Key, Chunk, Vec<TimeStamp>)>)
    -> Result<Vec<(Key, Chunk, Vec<TimeStamp>)>, Conflict>
{
    let mut map = HashMap::new();
    for (key, total, src) in vec.into_iter() {
        key.get_with(by, |x| x.to_string()).map(|kstr| {
            map.entry(kstr)
                .or_insert_with(Vec::new)
                .push((key, total, src));
        });
    }
    let mut res = Vec::new();
    for (key, mut vec) in map.into_iter() {
        let (_, datapoints, ts) = if vec.len() > 1 {
            try!(sum_series(&vec))
        } else {
            vec.pop().unwrap()
        };
        res.push((Key::from_pair(by, &key[..]), datapoints, ts));
    }
    return Ok(res);
}


fn sum_series(src: &Vec<(Key, Chunk, Vec<TimeStamp>)>)
    -> Result<(Key, Chunk, Vec<TimeStamp>), Conflict>
{
    use history::ChunkSet as S;
    use history::Chunk as C;
    assert!(src.len() > 1);
    let ts = src[0].2.clone();
    for &(_, _, ref nts) in &src[1..] {
        if &ts != nts {
            return Err(Conflict::CantSumTimestamps);
        }
    }
    let data_points = ts.len();
    let chunk = match
        ChunkSet::merge(src.iter().map(|&(_, ref chunk, _)| chunk))
    {
        S::Empty => unreachable!(),
        S::Counters(lst) => C::Counter(lst.iter()
            .fold(vec![None; data_points], vec_sum)),
        S::Integers(lst) => C::Integer(lst.iter()
            .fold(vec![None; data_points], vec_sum)),
        S::Floats(lst) => C::Float(lst.iter()
            .fold(vec![None; data_points], vec_sum)),
        S::States(_) => return Err(Conflict::CantSumStates),
        S::Conflict => return Err(Conflict::Dissimilar),
    };
    Ok((Key::empty(), chunk, ts))
}

fn vec_sum<X:Add<X, Output=X>+Copy>(mut target: Vec<Option<X>>,
                                        source: &&Vec<Option<X>>)
    -> Vec<Option<X>>
{
    for i in 0..target.len() {
        match (&mut target[i], &source[i]) {
            (&mut Some(ref mut x), &Some(ref y)) => *x = *x + *y,
            (x @ &mut None, y) => *x = *y,
            (&mut Some(_), &None) => {}
        }
    }
    return target;
}

fn sum_iter<A:Add<A,Output=A>, I:Iterator<Item=A>>(mut iter: I) -> A {
    let mut x = iter.next().unwrap();
    for y in iter {
        x = x + y;
    }
    return x;
}

fn sum_tip(mut src: Vec<(Key, Value, TimeSlice)>) -> Dataset {
    use history::ValueSet as S;
    use values::Value as V;

    // For now assuming that timestamps are equal
    // for different timestamps we need more complex algo
    if src.len() == 0 {
        return Dataset::Empty;
    }
    if src.len() == 1 {
        let (k, c, t) = src.pop().unwrap();
        return Dataset::SingleTip(k, c, t);
    }
    for &(_, _, ref nts) in &src[1..] {
        if &src[0].2 != nts {
            return Dataset::Incompatible(Conflict::CantSumTimestamps);
        }
    }
    let value = match
        ValueSet::merge(src.iter().map(|&(_, ref chunk, _)| chunk))
    {
        S::Empty => return Dataset::Empty,
        S::Counters(lst) => V::Counter(sum_iter(lst.into_iter())),
        S::Integers(lst) => V::Integer(sum_iter(lst.into_iter())),
        S::Floats(lst) => V::Float(sum_iter(lst.into_iter())),
        S::States(_) => return Dataset::Incompatible(Conflict::CantSumStates),
        S::Conflict => return Dataset::Incompatible(Conflict::Dissimilar),
    };
    Dataset::SingleTip(Key::empty(), value, src.pop().unwrap().2)
}
