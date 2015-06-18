use std::collections::HashMap;

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

pub fn query(query: &Query, stats: &Stats) -> Result<Vec<String>, Error> {
    debug!("Query {:?}", query);
    return Err(Error("Not implemented"));
}
