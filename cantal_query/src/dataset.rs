use std::collections::HashMap;

use history::{Key, Chunk};
use values::Value;


#[derive(Debug)]
pub enum Dataset {
    SingleSeries(Chunk),
    MultiSeries(Vec<(Key, Chunk)>),
    SingleTip(Key, Value),
    MultiTip(Vec<(Key, Value)>),
    Chart(HashMap<String, usize>),
    // TODO(tailhook) multi-chart
}

// Keep in sync with query::rule::Expectation
probor_enum_encoder_decoder!(Dataset {
    #100 SingleSeries(data #1),
    #101 MultiSeries(data #1),
    #200 SingleTip(key #1, value #3),
    #201 MultiTip(data #1),
    #300 Chart(data #1),
});

