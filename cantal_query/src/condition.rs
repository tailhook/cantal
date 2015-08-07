use std::hash::{Hash, Hasher};

use regex::Regex;
use rustc_serialize::{Encodable, Decodable, Encoder, Decoder};


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Condition {
    Eq(String, String),
    NotEq(String, String),
    RegexLike(String, Regex),
    And(Box<Condition>, Box<Condition>),
    Or(Box<Condition>, Box<Condition>),
    Not(Box<Condition>),
    Has(String),
}

impl Decodable for Condition {
    fn decode<D: Decoder>(d: &mut D) -> Result<Condition, D::Error> {
        use self::Condition::*;
        d.read_seq(|d, len| {
            let s = try!(d.read_str());
            let norm_len = if s == "not" || s == "has" { 2 } else { 3 };
            if norm_len != len {
                return Err(d.error("Bad tuple length condition"));
            }
            match &s[..] {
                "not" => Ok(Not(Box::new(try!(Decodable::decode(d))))),
                "has" => Ok(Has(try!(Decodable::decode(d)))),
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

impl Encodable for Condition {
    fn encode<E: Encoder>(&self, e: &mut E) -> Result<(), E::Error> {
        use self::Condition::*;
        let len = match self {
            &Not(_) => 1,
            &Has(_) => 1,
            _ => 2,
        };
        e.emit_seq(len, |e| {
            try!(e.emit_seq_elt(0, |e| (match self {
                &Eq(_, _) => "eq",
                &NotEq(_, _) => "not-eq",
                &RegexLike(_, _) => "regex-like",
                &And(_, _) => "and",
                &Or(_, _) => "or",
                &Not(_) => "not",
                &Has(_) => "has",
            }).encode(e)));
            try!(e.emit_seq_elt(1, |e| match self {
                &Eq(ref x, _) => x.encode(e),
                &NotEq(ref x, _) => x.encode(e),
                &RegexLike(ref x, _) => x.encode(e),
                &And(ref x, _) => x.encode(e),
                &Or(ref x, _) => x.encode(e),
                &Not(ref x) => x.encode(e),
                &Has(ref x) => x.encode(e),
            }));
            if len >= 2 {
                try!(e.emit_seq_elt(2, |e| match self {
                    &Eq(_, ref x) => x.encode(e),
                    &NotEq(_, ref x) => x.encode(e),
                    &RegexLike(_, ref x) => x.as_str().encode(e),
                    &And(_, ref x) => x.encode(e),
                    &Or(_, ref x) => x.encode(e),
                    _ => unreachable!(),
                }));
            }
            Ok(())
        })
    }
}

impl Hash for Condition {
    fn hash<H>(&self, s: &mut H) where H: Hasher {
        use self::Condition::*;
        match self {
            &Eq(ref a, ref b) => { "eq".hash(s); a.hash(s); b.hash(s); },
            &NotEq(ref a, ref b) => { "not-eq".hash(s); a.hash(s); b.hash(s); },
            &RegexLike(ref a, ref b) => {
                "regex-like".hash(s);
                a.hash(s);
                b.as_str().hash(s);
            },
            &And(ref a, ref b) => { "and".hash(s); a.hash(s); b.hash(s); }
            &Or(ref a, ref b) => { "or".hash(s); a.hash(s); b.hash(s); }
            &Not(ref a) => { "not".hash(s); a.hash(s); }
            &Has(ref a) => { "has".hash(s); a.hash(s); }
        }
    }
}
