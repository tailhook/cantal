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
    /// Note: caller must ensure that order is Okay
    fn from_iter<'x, I>(pairs: I) -> Key
        where I :Iterator<Item=(&'x str, &'x str)>+ExactSizeIterator
    {
        let mut e = Encoder::new(Vec::new());
        e.object(pairs.len()).unwrap();
        for (k, v) in pairs {
            e.text(&k).unwrap();
            // TODO(tailhook) optimize numbers
            e.text(&v).unwrap();
        }
        Key(e.into_writer().into_boxed_slice())
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
        return &self.0[..]
    }

    pub fn get_with<'x, F, T>(&'x self, name: &str, f: F) -> Option<T>
        where F: FnOnce(&str) -> T
    {
        let mut d = Decoder::new(Config::default(), Cursor::new(&self.0[..]));
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
    }

    pub fn intersection<'x, I:Iterator<Item=&'x Key>>(_iter: I) -> Key {
        let enc = Encoder::new(Vec::new());
        let cnt = 0;
        /*
        let decoders = iter
            .map(|&k| Decoder::new(Config::default(), Cursor::new(&k.0[..])))
            .collect::<Vec<_>>();

        for c in decoders {
            // TODO(tailhook) implement real intersection
        }
        */

        let mut buf = enc.into_writer();
        let bytes;
        let mut lenbuf = [0u8; 8];
        {
            let mut enc = Encoder::new(Cursor::new(&mut lenbuf[..]));
            enc.object(cnt).unwrap();
            bytes = enc.into_writer().position() as usize;
        }
        // It's almost never more than 1 byte so we don't care it's
        // theoretically very slow
        for &b in lenbuf[..bytes].iter().rev() {
            buf.insert(0, b);
        }
        return Key(buf.into_boxed_slice());
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
            Ok(Some(Key(value.into_boxed_slice())))
        }
    }

    impl Encodable for Key {
        fn encode<W:Output>(&self, e: &mut Encoder<W>)
            -> Result<(), EncodeError>
        {
            e.bytes(&self.0[..])
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
            let mut d = Decoder::new(Config::default(),
                Cursor::new(&self.0[..]));
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
            Key(self.0.to_vec().into_boxed_slice())
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
        assert_eq!(&key.0[..],
            &b"\xa3fmetricdtestcpidd1234czooebasic"[..]);
    }

    fn intersection() {
        let key1 = Key::from_json(&json::Json::from_str(
            r#"{"a": 1, "b": 2, "c": 3, "d": 4}"#));
        let key2 = Key::from_json(&json::Json::from_str(
            r#"{"a": 1, "b": 2, "c": 3, "e": 4}"#));
        let key3 = Key::from_json(&json::Json::from_str(
            r#"{"a": 5, "b": 2, "c": 3, "e": 4}"#));
        let key = Key::intersection(key1, key2, key3);
        assert_eq!(&key.0[..],
            &b"\xa2ab\x02ac\x03"[..]);
    }
}
