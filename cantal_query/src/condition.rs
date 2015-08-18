use regex::Regex;
use history::Key;

/// A shim type to deserialize regex and hash it
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct RegexWrap(Regex);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Condition {
    Eq(String, String),
    NotEq(String, String),
    RegexLike(String, RegexWrap),
    And(Box<Condition>, Box<Condition>),
    Or(Box<Condition>, Box<Condition>),
    Not(Box<Condition>),
    Has(String),
}

probor_enum_encoder_decoder!(Condition {
    #0 Eq(left #1, right #2),
    #1 NotEq(left #1, right #2),
    #2 RegexLike(left #1, right #2),
    #3 And(left #1, right #2),
    #4 Or(left #1, right #2),
    #5 Not(val #1),
    #6 Has(field #1),
});

json_enum_decoder!(Condition {
    Eq(left, right),
    NotEq(left, right),
    RegexLike(left, right),
    And(left, right),
    Or(left, right),
    Not(val),
    Has(field),
});

impl Condition {
    pub fn matches(&self, key: &Key) -> bool {
        use self::Condition::*;
        match self {
            &Eq(ref name, ref value) => {
                key.get_with(name, |x| x == value).unwrap_or(false)
            },
            &NotEq(ref name, ref value) => {
                key.get_with(name, |x| x != value).unwrap_or(false)
            }
            &RegexLike(ref name, ref regex) => {
                key.get_with(name, |x| regex.is_match(x)).unwrap_or(false)
            }
            &And(ref a, ref b) => a.matches(key) && b.matches(key),
            &Or(ref a, ref b) => a.matches(key) || b.matches(key),
            &Not(ref x) => !x.matches(key),
            &Has(ref name) => key.get_with(name, |_| ()).is_some(),
        }
    }
}

mod regex_wrap {
    use std::ops::Deref;
    use std::hash::{Hash, Hasher};
    use super::RegexWrap;
    use rustc_serialize;
    use probor;
    use regex::Regex;

    impl Hash for RegexWrap {
        fn hash<H>(&self, s: &mut H) where H: Hasher {
            self.as_str().hash(s);
        }
    }

    impl Deref for RegexWrap {
        type Target = Regex;
        fn deref<'x>(&'x self) -> &'x Regex {
            &self.0
        }
    }

    impl probor::Encodable for RegexWrap {
        fn encode<W:probor::Output>(&self, e: &mut probor::Encoder<W>)
            -> Result<(), probor::EncodeError>
        {
            self.0.encode(e)
        }
    }

    impl probor::Decodable for RegexWrap {
        fn decode_opt<R:probor::Input>(e: &mut probor::Decoder<R>)
            -> Result<Option<RegexWrap>, probor::DecodeError>
        {
            probor::Decodable::decode_opt(e).map(|x| x.map(RegexWrap))
        }
    }

    impl rustc_serialize::Decodable for RegexWrap {
        fn decode<D: ::rustc_serialize::Decoder>(d: &mut D)
            -> Result<RegexWrap, D::Error>
        {
            match d.read_str() {
                Ok(x) => match Regex::new(&x) {
                    Ok(r) => Ok(RegexWrap(r)),
                    Err(_) => Err(d.error("Invalid regex")),
                },
                Err(e) => Err(e),
            }
        }
    }
}
