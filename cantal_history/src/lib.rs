#![crate_name="cantal_history"]

#[macro_use] extern crate log;
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

pub use backlog::Backlog;
pub use tip::Tip;
pub use merge::ValueSet;
pub use chunk::HistoryChunk;


#[derive(Debug)]
pub struct History {
    /// Values that are kept as fine-grained as possible (2-second interval)
    pub fine: Backlog,
    /// Values that need only last value to be stored
    pub tip: Tip,
}

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

