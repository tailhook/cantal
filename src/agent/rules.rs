use regex::Regex;
use std::collections::{HashMap, BTreeMap};
use rustc_serialize::json::Json;
use rustc_serialize::{Decodable, Decoder};

use super::aio::http;
use super::stats::{Stats, Key};

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
    Sum,
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

fn query_tip(rule: &Rule, stats: &Stats) -> Result<Json, Error> {
    for (&Key(ref key), ref value) in stats.history.tip.iter() {
        if match_cond(key, &rule.condition) {
        }
    }
    return Ok(Json::Object(BTreeMap::new()));
}

fn query_fine(rule: &Rule, stats: &Stats) -> Result<Json, Error> {
    for (&Key(ref key), ref value) in stats.history.fine.iter() {
        if match_cond(key, &rule.condition) {
            println!("MATCHED {:?}", key);
        }
    }
    return Ok(Json::Object(BTreeMap::new()));
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
