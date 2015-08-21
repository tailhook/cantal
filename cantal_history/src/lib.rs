#![crate_name="cantal_history"]

#[macro_use] extern crate log;
#[macro_use] extern crate probor;
extern crate cbor;
extern crate num;
extern crate rustc_serialize as serialize;
extern crate cantal_values as values;
extern crate byteorder;

mod key;
mod deltabuf;
mod chunk;
mod backlog;
mod tip;
mod merge;
mod serde;

pub use backlog::{Backlog, Value};
pub use tip::Tip;
pub use merge::{ChunkSet, ValueSet};
pub use chunk::HistoryChunk as Chunk;
pub use serde::VersionInfo;

pub type TimeStamp = u64;  // Milliseconds
pub type TimeDelta = u32;  // Milliseconds
pub type SnapTime = (TimeStamp, TimeDelta);

#[derive(Debug)]
pub struct History {
    /// Values that are kept as fine-grained as possible (2-second interval)
    pub fine: Backlog,
    /// Values that need only last value to be stored
    pub tip: Tip,
}

// Named fields are ok since we don't store lots of History objects
probor_struct_encoder_decoder!(History {
    fine => (),
    tip => (),
});

///
/// This contains CBOR-encoded key-value pairs
#[derive(Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Key(Box<[u8]>);

impl History {
    pub fn new() -> History {
        return History {
            tip: Tip::new(),
            fine: Backlog::new(),
        }
    }
    pub fn truncate_by_time(&mut self, tstamp: u64) {
        self.fine.truncate_by_time(tstamp);
        self.tip.truncate_by_time(tstamp);
    }
}



#[cfg(test)]
mod test {
    use std::io::Cursor;
    use {History, Key, ValueSet};
    use values::Value::{Counter, State};
    use std::collections::{HashMap, HashSet};
    use probor::{Encodable, Encoder, decode, Config, Decoder};

    #[test]
    fn roundtrip() {
        let mut h = History::new();
        h.fine.push((1000, 10), vec![
            (&Key::metric("test1"), &Counter(10)),
            (&Key::metric("test2"), &Counter(20)),
        ].into_iter());
        h.fine.push((2000, 10), vec![
            (&Key::metric("test2"), &Counter(20)),
            (&Key::metric("test3"), &Counter(30)),
        ].into_iter());
        h.tip.push((2000, 10), vec![
            (&Key::metric("st1"), &State(1500, "hello".to_string())),
            (&Key::metric("st2"), &Counter(30)),
        ].into_iter());
        let mut e = Encoder::new(Vec::new());
        h.encode(&mut e).unwrap();
        let h: History = decode(&mut Decoder::new(Config::default(),
            Cursor::new(&e.into_writer()[..]))).unwrap();
    }
}
