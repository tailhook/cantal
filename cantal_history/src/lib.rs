#![crate_name="cantal_history"]

#[macro_use] extern crate log;
extern crate cbor;
extern crate num;
extern crate rustc_serialize as serialize;
extern crate cantal_values as values;

mod mem;
mod key;
mod deltabuf;
mod chunk;
mod backlog;
mod tip;
mod merge;

pub use key::Key;
pub use backlog::Backlog;
pub use tip::Tip;
pub use merge::ValueSet;
pub use mem::History;
pub use chunk::HistoryChunk;
