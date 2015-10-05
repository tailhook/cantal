use std::mem::size_of_val;
use std::io::Cursor;
use std::collections::BTreeMap;
use std::marker::PhantomData;
use std::iter::Peekable;

use cbor::{Encoder, Decoder, Config};
use serialize::json::Json;

use Key;

struct Merge<'a, A, B>(usize, Peekable<A>, Peekable<B>, PhantomData<&'a A>)
    where A: Iterator<Item=(&'a str, &'a str)> + 'a,
          B: Iterator<Item=(&'a str, &'a str)> + 'a;

impl<'a, A, B> Iterator for Merge<'a, A, B>
    where A: Iterator<Item=(&'a str, &'a str)>,
          B: Iterator<Item=(&'a str, &'a str)>
{
    type Item = (&'a str, &'a str);
    fn next(&mut self) -> Option<(&'a str, &'a str)> {
        match (self.1.peek(), self.2.peek()) {
            (_, None) => self.1.next(),
            (None, _) => self.2.next(),
            (Some(&(a, _)), Some(&(b, _))) if a == b => {
                // latter overrides former
                self.1.next();
                self.2.next()
            }
            (Some(&(a, _)), Some(&(b, _))) if a < b    => self.1.next(),
            (Some(&(_, _)), Some(&(_, _))) /* a > b */ => self.2.next(),
        }
    }
}

impl<'a, A, B> ExactSizeIterator for Merge<'a, A, B>
    where A: Iterator<Item=(&'a str, &'a str)>,
          B: Iterator<Item=(&'a str, &'a str)>
{
    fn len(&self) -> usize {
        self.0
    }
}


impl Key {
    /// Size of key in bytes, for debugging
    pub fn size(&self) -> usize {
        size_of_val(self) + self.0.as_ref().map(|x| x.len()).unwrap_or(0)
    }
    /// Note: caller must ensure that order is Okay
    fn from_iter<'x, I>(pairs: I) -> Key
        where I :Iterator<Item=(&'x str, &'x str)>+ExactSizeIterator
    {
        if pairs.len() == 0 {
            return Key(None);
        }
        let mut e = Encoder::new(Vec::new());
        e.object(pairs.len()).unwrap();
        for (k, v) in pairs {
            e.text(&k).unwrap();
            // TODO(tailhook) optimize numbers
            e.text(&v).unwrap();
        }
        Key(Some(e.into_writer().into_boxed_slice()))
    }
    /// Creates a key json object with additional pairs added
    ///
    /// Note pairs should be sorted
    pub fn from_json(json: &Json, pairs: &[(&str, &str)]) -> Result<Key, ()> {
        debug_assert!(pairs.iter().zip(pairs.iter().skip(1))
            .all(|(&(a, _), &(b, _))| a < b));
        if let &Json::Object(ref obj) = json {
            let btree: &BTreeMap<_, _> = obj;  // assert it's BTree
                                               // because we care about order
            let mut errors = 0;
            let num = btree.len() + pairs.len() -
                pairs.iter().filter(|&&(x, _)| btree.contains_key(x)).count();
            let res = Key::from_iter(Merge(num,
                pairs.iter().cloned().peekable(),
                btree.iter().map(|(ref k, v)| {
                    match v {
                        &Json::String(ref val) => {
                            (&k[..], &val[..])
                        }
                        // TODO(tailhook) probably support numbers
                        _ => {
                            errors += 1;
                            (&k[..], "")
                        }
                    }
                }).peekable(),
                PhantomData,
                ));
            if errors > 0 {
                return Err(());
            }
            return Ok(res);
        } else {
            return Err(());
        }
    }
    /// Creates a key from list of pairs, asserts that pairs are in right
    /// order. This method is inteded to be used with literal values put in
    /// the code, so it should be easy to put them in the right order
    pub fn pairs(pairs: &[(&str, &str)]) -> Key {
        debug_assert!(pairs.iter().zip(pairs.iter().skip(1))
            .all(|(&(a, _), &(b, _))| a < b));
        Key::from_iter(pairs.iter().cloned())
    }
    pub fn from_pair(key: &str, val: &str) -> Key {
        Key::from_iter([(key, val)].iter().cloned())
    }
    pub fn metric(metric: &str) -> Key {
        Key::from_iter([("metric", metric)].iter().cloned())
    }

    pub fn as_bytes<'x>(&'x self) -> &'x [u8] {
        self.0.as_ref().map(|x| &x[..]).unwrap_or(b"")
    }

    pub fn get_with<'x, F, T>(&'x self, name: &str, f: F) -> Option<T>
        where F: FnOnce(&str) -> T
    {
        self.0.as_ref().and_then(|b| {
            let mut d = Decoder::new(Config::default(), Cursor::new(&b[..]));
            let num = d.object().unwrap();
            for _ in 0..num {
                if d.text_borrow().unwrap() == name {
                    // TODO(tailhook) other types may work in future
                    return Some(f(d.text_borrow().unwrap()));
                } else {
                    d.skip().unwrap();
                }
            }
            return None;
        })
    }

    pub fn empty() -> Key {
        Key(None)
    }

}

mod serde {
    use std::io::Cursor;
    use probor::{Decodable, Decoder, DecodeError, Input, Config};
    use probor::{Encodable, Encoder, EncodeError, Output};
    use Key;

    fn validate_key(val: &[u8]) -> Result<(), &'static str> {
        let mut d = Decoder::new(Config::default(), Cursor::new(val));
        let num = try!(d.object().map_err(|_| "Invalid key"));
        for _ in 0..num {
            // TODO(tailhook) other types may work in future
            try!(d.text_borrow().map_err(|_| "Invalid key"));
            try!(d.text_borrow().map_err(|_| "Invalid key"));
        }
        if d.into_reader().position() as usize != val.len() {
            return Err("Invalid key: extra data");
        }
        return Ok(());
    }

    impl Decodable for Key {
        fn decode_opt<R:Input>(d: &mut Decoder<R>)
            -> Result<Option<Self>, DecodeError>
        {
            let value = try!(d.bytes().map_err(|e|
                DecodeError::WrongType("bytes expected", e)));
            try!(validate_key(&value[..]).map_err(|e|
                DecodeError::WrongValue(e)));
            Ok(Some(Key(Some(value.into_boxed_slice()))))
        }
    }

    impl Encodable for Key {
        fn encode<W:Output>(&self, e: &mut Encoder<W>)
            -> Result<(), EncodeError>
        {
            e.bytes(self.as_bytes())
        }
    }
}

mod std_trait {
    use std::fmt::{Debug, Formatter, Error};
    use std::io::Cursor;
    use cbor::{Decoder, Config};
    use Key;

    impl Debug for Key {
        fn fmt(&self, f: &mut Formatter) -> Result<(), Error>
        {
            let b = if let Some(ref b) = self.0 { b } else {
                try!(write!(f, "Key {{}}"));
                return Ok(());
            };
            let mut d = Decoder::new(Config::default(), Cursor::new(&b[..]));
            try!(write!(f, "Key {{"));
            let num = try!(d.object()
                .map_err(|_| Error));
            for _ in 0..num {
                // TODO(tailhook) other types may work in future
                try!(write!(f, "{}: ",
                    try!(d.text_borrow().map_err(|_| Error))));
                try!(write!(f, "{}",
                    try!(d.text_borrow().map_err(|_| Error))));
            }
            try!(write!(f, "}}"));
            Ok(())
        }
    }

    impl Clone for Key {
        fn clone(&self) -> Key {
            // Unfortunately Box<[u8]> doesn't support Clone
            Key(self.0.as_ref().map(|x| x.to_vec().into_boxed_slice()))
        }
    }
}

#[cfg(test)]
mod test {
    use serialize::json;
    use Key;

    fn from_json() {
        let key = Key::from_json(&json::Json::from_str(
                r#"{"metric": "test", "zoo": "basic"}"#
                ).unwrap(),
                &[("pid", "1234")]
            ).unwrap();
        assert_eq!(&key.0.unwrap()[..],
            &b"\xa3fmetricdtestcpidd1234czooebasic"[..]);
    }
}
