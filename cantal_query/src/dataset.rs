use std::collections::HashMap;

use history::{Key, Chunk, TimeStamp};
use values::Value;


#[derive(Debug)]
pub enum Dataset {
    SingleSeries(Key, Chunk, Vec<TimeStamp>),
    MultiSeries(Vec<(Key, Chunk, Vec<TimeStamp>)>),
    SingleTip(Key, Value),
    MultiTip(Vec<(Key, Value)>),
    Chart(HashMap<String, usize>),
    // TODO(tailhook) multi-chart
}

// Keep in sync with query::rule::Expectation
probor_enum_encoder_decoder!(Dataset {
    #100 SingleSeries(key #1, data #2, timestamps #3),
    #101 MultiSeries(data #1),
    #200 SingleTip(key #1, value #3),
    #201 MultiTip(data #1),
    #300 Chart(data #1),
});

