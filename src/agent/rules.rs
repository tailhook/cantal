use std::ops::{Add, Sub};
use std::iter::{repeat};
use std::collections::{HashMap, BTreeMap};

use regex::Regex;
use rustc_serialize::json::{Json, ToJson};
use rustc_serialize::{Decodable, Decoder};

use super::aio::http;
use super::stats::{Stats, Key};
use super::history::{merge};

pub struct Error(&'static str);

impl From<Error> for http::Error {
    fn from(err: Error) -> http::Error {
        return http::Error::BadRequest(err.0);
    }
}


#[derive(RustcEncodable, RustcDecodable, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    Tip,
    Fine,
    Coarse,
}

#[derive(RustcEncodable, RustcDecodable, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Aggregation {
    None,
    CasualSum,  // does ignore absent data points, treating them as zero
}

#[derive(RustcEncodable, RustcDecodable, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Load {
    Raw,
    Rate,
    Tip,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Condition {
    Eq(String, String),
    NotEq(String, String),
    RegexLike(String, Regex),
    And(Box<Condition>, Box<Condition>),
    Or(Box<Condition>, Box<Condition>),
    Not(Box<Condition>),
}

impl Decodable for Condition {
    fn decode<D: Decoder>(d: &mut D) -> Result<Condition, D::Error> {
        use self::Condition::*;
        d.read_seq(|d, len| {
            let s = try!(d.read_str());
            let norm_len = if s == "not" { 2 } else { 3 };
            if norm_len != len {
                return Err(d.error("Bad tuple length condition"));
            }
            match &s[..] {
                "not" => Ok(Not(Box::new(try!(Decodable::decode(d))))),
                "and" => Ok(And(
                    Box::new(try!(Decodable::decode(d))),
                    Box::new(try!(Decodable::decode(d))),
                    )),
                "or" => Ok(Or(
                    Box::new(try!(Decodable::decode(d))),
                    Box::new(try!(Decodable::decode(d))),
                    )),
                "eq" => Ok(Eq(try!(d.read_str()), try!(d.read_str()))),
                "not-eq" => Ok(NotEq(try!(d.read_str()), try!(d.read_str()))),
                "regex-like" => Ok(RegexLike(
                    try!(d.read_str()),
                    try!(Regex::new(&try!(d.read_str())[..])
                         .map_err(|_| d.error("Error compiling regex"))))),
                _ => Err(d.error("Bad condition type")),
            }
        })
    }
}

#[derive(RustcDecodable, Debug, Clone)]
pub struct Rule {
    pub source: Source,
    pub condition: Condition,
    pub key: Vec<String>,
    pub aggregation: Aggregation,
    pub load: Load,
    pub limit: u32,
}

#[derive(RustcDecodable, Debug, Clone)]
pub struct Query {
    pub rules: HashMap<String, Rule>,
}

fn match_cond(key: &Key, cond: &Condition) -> bool {
    use self::Condition::*;
    match cond {
        &Eq(ref name, ref value) => key.get(name) == Some(value),
        &NotEq(ref name, ref value) => key.get(name) != Some(value),
        &RegexLike(ref name, ref regex)
        => regex.is_match(key.get(name).unwrap_or(&String::from(""))),
        &And(ref a, ref b) => match_cond(key, a) && match_cond(key, b),
        &Or(ref a, ref b) => match_cond(key, a) || match_cond(key, b),
        &Not(ref x) => !match_cond(key, x),
    }
}

fn query_tip(_rule: &Rule, _stats: &Stats) -> Result<Json, Error> {
    unimplemented!()
}

fn tree_insert(map: &mut BTreeMap<String, Json>, key: &[&str], val: Json) {
    use std::collections::btree_map::Entry::{Occupied, Vacant};
    if key.len() == 1 {
        map.insert(key[0].to_string(), val);
    } else {
        match map.entry(key[0].to_string()) {
                Occupied(mut x) => {
                    tree_insert(x.get_mut().as_object_mut().unwrap(),
                                &key[1..], val);
                }
                Vacant(x) => {
                    let mut m = BTreeMap::new();
                    tree_insert(&mut m, &key[1..], val);
                    x.insert(Json::Object(m));
                }
        }
    }
}

fn json_tree<'x, I>(iter: I) -> Json
    where I: Iterator<Item=(Vec<&'x str>, Json)>
{
    let mut res = BTreeMap::new();
    for (key, val) in iter {
        if key.len() == 0 {
            // assert!(iter.next().is_none());
            return val;
        }
        tree_insert(&mut res, &key[..], val);
    }
    return Json::Object(res);
}

fn take_tip<T:ToJson, I:Iterator<Item=Option<T>>>(items: Vec<I>, limit: u32)
    -> Json
{
    items.into_iter()
        .map(|x| x.take(limit as usize).find(Option::is_some).and_then(|x| x))
        .collect::<Vec<_>>().to_json()
}

fn take_raw<T:ToJson, I:Iterator<Item=Option<T>>>(items: Vec<I>, limit: u32)
    -> Json
{
    items.into_iter()
        .map(|x| x.take(limit as usize).collect::<Vec<_>>())
        .collect::<Vec<_>>().to_json()
}

fn take_rate<T:Sub<T, Output=T>+ToJson, I>(items: Vec<I>, limit: u32)
    -> Json
    where I: Iterator<Item=Option<T>> + Clone
{
    items.into_iter().map(|iter| {
        let mut pair = iter.clone();
        pair.next();
        iter.zip(pair)
            .map(|(new, old)| new.and_then(|new| old.map(|old| new - old)))
            .take(limit as usize)
            .collect::<Vec<_>>()
    }).collect::<Vec<_>>().to_json()
}
fn sum_tip<T, I>(items: Vec<I>, limit: u32, zero: T) -> Json
    where T: Add<T, Output=T> + ToJson,
          I: Iterator<Item=Option<T>>
{
    items.into_iter()
        .filter_map(|x| x.take(limit as usize)
                         .find(Option::is_some).and_then(|x| x))
        .fold(zero, |acc, val| acc + val).to_json()
}

fn sum_raw<T, I>(items: Vec<I>, limit: u32, zero: T) -> Json
    where T: Add<T, Output=T> + ToJson + Copy,
          I: Iterator<Item=Option<T>>
{
    let buf: Vec<_> = items.into_iter()
        .map(|x| x.take(limit as usize)
                  .map(|x| x.unwrap_or(zero))
                  .collect::<Vec<_>>())
        .collect();
    let rlen = buf.iter().map(Vec::len).max().unwrap_or(0);
    let zeros = repeat(zero).take(rlen).collect::<Vec<T>>();
    buf.into_iter()
        .fold(zeros, |mut acc, val| {
            for (i, v) in val.into_iter().enumerate() {
                acc[i] = acc[i] + v;
            }
            acc
        })
        .to_json()
}

fn sum_rate<T, I>(items: Vec<I>, limit: u32, zero: T)
    -> Json
    where I: Iterator<Item=Option<T>> + Clone,
          T: Sub<T, Output=T> + Add<T, Output=T> + ToJson + Copy
{
    let buf: Vec<_> = items.into_iter().map(|iter| {
        let mut pair = iter.clone();
        pair.next();
        iter.zip(pair)
            .map(|(new, old)|
                new.and_then(|new| old.map(|old| new - old))
                .unwrap_or(zero))
            .take(limit as usize)
            .collect::<Vec<_>>()
    }).collect();
    let rlen = buf.iter().map(Vec::len).max().unwrap_or(0);
    let zeros = repeat(zero).take(rlen).collect::<Vec<T>>();
    buf.into_iter()
        .fold(zeros, |mut acc, val| {
            for (i, v) in val.into_iter().enumerate() {
                acc[i] = acc[i] + v;
            }
            acc
        })
        .to_json()
}

fn query_fine(rule: &Rule, stats: &Stats) -> Result<Json, Error> {
    use self::Aggregation::*;
    use self::Load::*;
    use std::collections::hash_map::Entry::{Occupied, Vacant};
    let mut keys = HashMap::<_, Vec<_>>::new();
    for (ref key, _) in stats.history.fine.iter() {
        if match_cond(key, &rule.condition) {
            let target_key = rule.key.iter()
                             .map(|x| key.get(x).unwrap_or(""))
                             .collect::<Vec<_>>();
            match keys.entry(target_key) {
                Occupied(mut e) => { e.get_mut().push(key.clone()); }
                Vacant(e) => { e.insert(vec![key.clone()]); }
            };
        }
    }
    Ok(json_tree(keys.into_iter().map(|(key, hkeys)| {
        use super::history::Histories::*;
        let stream = merge(hkeys.iter()
               .filter_map(|key| stats.history.get_fine_history(key)));
        let json = match rule.aggregation {
            None => match rule.load {
                Tip => stream.map(|s| match s {
                    Empty => Json::Null,
                    Counters(x) => take_tip(x, rule.limit),
                    Integers(x) => take_tip(x, rule.limit),
                    Floats(x) => take_tip(x, rule.limit),
                }),
                Raw => stream.map(|s| match s {
                    Empty => Json::Null,
                    Counters(x) => take_raw(x, rule.limit),
                    Integers(x) => take_raw(x, rule.limit),
                    Floats(x) => take_raw(x, rule.limit),
                }),
                Rate => stream.map(|s| match s {
                    Empty => Json::Null,
                    Counters(x) => take_rate(x, rule.limit),
                    Integers(x) => take_rate(x, rule.limit),
                    Floats(x) => take_rate(x, rule.limit),
                }),
            },
            CasualSum => match rule.load {
                Tip => stream.map(|s| match s {
                    Empty => Json::Null,
                    Counters(x) => sum_tip(x, rule.limit, 0),
                    Integers(x) => sum_tip(x, rule.limit, 0),
                    Floats(x) => sum_tip(x, rule.limit, 0.),
                }),
                Raw => stream.map(|s| match s {
                    Empty => Json::Null,
                    Counters(x) => sum_raw(x, rule.limit, 0),
                    Integers(x) => sum_raw(x, rule.limit, 0),
                    Floats(x) => sum_raw(x, rule.limit, 0.),
                }),
                Rate => stream.map(|s| match s {
                    Empty => Json::Null,
                    Counters(x) => sum_rate(x, rule.limit, 0),
                    Integers(x) => sum_rate(x, rule.limit, 0),
                    Floats(x) => sum_rate(x, rule.limit, 0.),
                }),
            },
        };
        (key, json.unwrap_or(Json::Null))
    })))
}

pub fn query(query: &Query, stats: &Stats) -> Result<Json, Error> {
    let mut items = BTreeMap::new();
    for (name, rule) in query.rules.iter() {
        match rule.source {
            Source::Tip => {
                items.insert(name.clone(), try!(query_tip(rule, stats)));
            }
            Source::Fine => {
                items.insert(name.clone(), try!(query_fine(rule, stats)));
            }
            Source::Coarse => unimplemented!(),
        }
    }
    return Ok(Json::Object(items));
}

