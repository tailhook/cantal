use std::collections::BTreeMap;

use cbor::{Encoder};
use serialize::json::Json;

use Key;

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
    pub fn from_json(json: &Json) -> Result<Key, ()> {
        if let &Json::Object(ref obj) = json {
            let btree: &BTreeMap<_, _> = obj;  // assert it's BTree
                                               // because we care about order
            let mut errors = 0;
            let res = Key::from_iter(btree.iter().map(|(ref k, v)| {
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
            }));
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

