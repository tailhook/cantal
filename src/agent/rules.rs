use std::ops::Add;
use std::iter::repeat;
use std::collections::{HashMap, BTreeMap};

use regex::Regex;
use rustc_serialize::json::{Json, ToJson};
use rustc_serialize::{Decodable, Decoder};

use super::aio::http;
use super::stats::{Stats, Key};
use super::history::Value;

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

fn match_cond(key: &BTreeMap<String, String>, cond: &Condition) -> bool {
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

fn tree_insert(map: &mut BTreeMap<String, Json>, key: &[&String], val: Json) {
    use std::collections::btree_map::Entry::{Occupied, Vacant};
    if key.len() == 1 {
        map.insert(key[0].clone(), val);
    } else {
        match map.entry(key[0].clone()) {
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
    where I: Iterator<Item=(Vec<&'x String>, Json)>
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

fn query_fine(rule: &Rule, stats: &Stats) -> Result<Json, Error> {
    use self::Aggregation::*;
    use self::Load::*;
    use std::collections::hash_map::Entry::{Occupied, Vacant};
    let dummy = String::from("");
    let mut keys = HashMap::<_, Vec<_>>::new();
    for (ref k, _) in stats.history.fine.iter() {
        let ref key = k.0;
        if match_cond(key, &rule.condition) {
            let target_key = rule.key.iter()
                             .map(|x| key.get(x).unwrap_or(&dummy))
                             .collect::<Vec<_>>();
            match keys.entry(target_key) {
                Occupied(mut e) => { e.get_mut().push(k.clone()); }
                Vacant(e) => { e.insert(vec![k.clone()]); }
            };
        }
    }
    Ok(json_tree(keys.into_iter().map(|(key, hkeys)| {
        (key, match rule.aggregation {
            None => match rule.load {
                Tip => unimplemented!(),
                Raw => unimplemented!(),
                Rate => unimplemented!(),
            },
            CasualSum => match rule.load {
                Tip => unimplemented!(),
                Raw => unimplemented!(),
                Rate => unimplemented!(),
            },
        })
    })))
}

pub fn query(query: &Query, stats: &Stats) -> Result<Json, Error> {
    debug!("Query {:?}", query);
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

