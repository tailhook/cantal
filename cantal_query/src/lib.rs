#![crate_name="cantal_query"]

extern crate log;
extern crate rustc_serialize;
extern crate regex;
extern crate num;
extern crate cantal_history as history;
extern crate cantal_values as values;
#[macro_use] extern crate probor;

#[macro_use] mod jsondecoder;
mod condition;
mod rule;
mod dataset;
mod query;
mod functions;

pub use condition::Condition;
pub use rule::{Source, Filter, Extract, Rule};
pub use rule::{MetricKind, UndefFilter, Function};
pub use dataset::{Dataset, Conflict, TimeSlice};
pub use query::query_history;
