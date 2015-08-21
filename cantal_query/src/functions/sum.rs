use std::ops::Add;

use history::{Key, Chunk, ChunkSet, ValueSet, TimeStamp};
use values::Value;
use {Dataset, Conflict, TimeSlice};

pub fn sum(src: Dataset) -> Dataset {
    use Dataset::*;
    match src {
        MultiSeries(vec) => sum_series(vec),
        MultiTip(vec) => sum_tip(vec),
        src @ SingleSeries(_, _, _) => src,
        src @ SingleTip(_, _, _) => src,
        src @ Incompatible(_) => src,
        Chart(_) => Incompatible(Conflict::CantSumChart),
        Empty => Empty,
    }
}


pub fn sum_series(mut src: Vec<(Key, Chunk, Vec<TimeStamp>)>) -> Dataset {
    use history::ChunkSet as S;
    use history::Chunk as C;
    // For now assuming that timestamps are equal
    // for different timestamps we need more complex algo
    if src.len() == 0 {
        return Dataset::Empty;
    }
    if src.len() == 1 {
        let (k, c, t) = src.pop().unwrap();
        return Dataset::SingleSeries(k, c, t);
    }
    for &(_, _, ref nts) in &src[1..] {
        if &src[0].2 != nts {
            return Dataset::Incompatible(Conflict::CantSumTimestamps);
        }
    }
    let data_points = src[0].2.len();
    let chunk = match ChunkSet::merge(src.iter().map(|&(_, ref chunk, _)| chunk)) {
        S::Empty => return Dataset::Empty,
        S::Counters(lst) => C::Counter(lst.iter()
            .fold(vec![None; data_points], vec_sum)),
        S::Integers(lst) => C::Integer(lst.iter()
            .fold(vec![None; data_points], vec_sum)),
        S::Floats(lst) => C::Float(lst.iter()
            .fold(vec![None; data_points], vec_sum)),
        S::States(_) => return Dataset::Incompatible(Conflict::CantSumStates),
        S::Conflict => return Dataset::Incompatible(Conflict::CantSumDissimilar),
    };
    let key = Key::intersection(src.iter().map(|&(ref k, _, _)| k));
    Dataset::SingleSeries(key, chunk, src.pop().unwrap().2)
}

pub fn vec_sum<X:Add<X, Output=X>+Copy>(mut target: Vec<Option<X>>,
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

pub fn sum_iter<A:Add<A,Output=A>, I:Iterator<Item=A>>(mut iter: I) -> A {
    let mut x = iter.next().unwrap();
    for y in iter {
        x = x + y;
    }
    return x;
}

pub fn sum_tip(mut src: Vec<(Key, Value, TimeSlice)>) -> Dataset {
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
    let value = match ValueSet::merge(src.iter().map(|&(_, ref chunk, _)| chunk)) {
        S::Empty => return Dataset::Empty,
        S::Counters(lst) => V::Counter(sum_iter(lst.into_iter())),
        S::Integers(lst) => V::Integer(sum_iter(lst.into_iter())),
        S::Floats(lst) => V::Float(sum_iter(lst.into_iter())),
        S::States(_) => return Dataset::Incompatible(Conflict::CantSumStates),
        S::Conflict => return Dataset::Incompatible(Conflict::CantSumDissimilar),
    };
    let key = Key::intersection(src.iter().map(|&(ref k, _, _)| k));
    Dataset::SingleTip(key, value, src.pop().unwrap().2)
}
