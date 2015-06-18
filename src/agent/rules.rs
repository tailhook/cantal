use regex::Regex;
use std::collections::{HashMap, BTreeMap};
use rustc_serialize::json::Json;

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
}

#[derive(RustcEncodable, RustcDecodable, Debug, Clone, PartialEq, Eq)]
pub enum Condition {
    Eq(String, String),
    NotEq(String, String),
    RegexLike(String, String),
    And(Box<Condition>, Box<Condition>),
    Or(Box<Condition>, Box<Condition>),
    Not(Box<Condition>),
}


#[derive(RustcEncodable, RustcDecodable, Debug, Clone)]
pub struct Rule {
    pub source: Source,
    pub condition: Condition,
    pub key: Vec<String>,
    pub aggregation: Aggregation,
    pub load: Load,
    pub limit: u32,
}

#[derive(RustcEncodable, RustcDecodable, Debug, Clone)]
pub struct Query {
    pub rules: HashMap<String, Rule>,
}

fn match_cond(key: &BTreeMap<String, String>, cond: &Condition) -> bool {
    use self::Condition::*;
    match cond {
        &Eq(ref name, ref value) => key.get(name) == Some(value),
        &NotEq(ref name, ref value) => key.get(name) != Some(value),
        // TODO(tailhook) implement regex compilation
        &RegexLike(ref name, ref value)
        => Regex::new(value).unwrap()  // TODO(tailhook) fix unwrap
            .is_match(key.get(name).unwrap_or(&String::from(""))),
        &And(ref a, ref b) => match_cond(key, a) && match_cond(key, b),
        &Or(ref a, ref b) => match_cond(key, a) || match_cond(key, b),
        &Not(ref x) => match_cond(key, x),
    }
}

fn query_tip(rule: &Rule, stats: &Stats) -> Result<Json, Error> {
    for (&Key(ref key), ref value) in stats.history.tip.iter() {
        if match_cond(key, &rule.condition) {
            println!("MATCHED {:?}", key);
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
