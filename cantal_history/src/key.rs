use std::collections::BTreeMap;
use std::marker::PhantomData;
use std::iter::Peekable;

use cbor::{Encoder};
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
                try!(write!(f, "{}: {}",
                    try!(d.text().map_err(|_| Error)),
                    try!(d.text().map_err(|_| Error))));
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
}
