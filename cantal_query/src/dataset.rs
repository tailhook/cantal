use std::collections::HashMap;

use history::{Key, Chunk, TimeStamp};
use values::Value;

pub type TimeSlice = (TimeStamp, TimeStamp);


#[derive(Debug)]
pub enum Conflict {
    CantSumChart,
    CantSumDissimilar,
    CantSumTimestamps,
    CantSumStates,
}

probor_enum_encoder_decoder!(Conflict {
    #100 CantSumChart(),
    #101 CantSumDissimilar(),
    #102 CantSumTimestamps(),
    #103 CantSumStates(),
});

#[derive(Debug)]
pub enum Dataset {
    SingleSeries(Key, Chunk, Vec<TimeStamp>),
    MultiSeries(Vec<(Key, Chunk, Vec<TimeStamp>)>),
    SingleTip(Key, Value, TimeSlice),
    MultiTip(Vec<(Key, Value, TimeSlice)>),
    Chart(HashMap<String, usize>),
    // TODO(tailhook) multi-chart
    Empty,
    Incompatible(Conflict),
}

// Keep in sync with query::rule::Expectation
probor_enum_encoder_decoder!(Dataset {
    #100 SingleSeries(key #1, data #2, timestamps #3),
    #101 MultiSeries(data #1),
    #200 SingleTip(key #1, value #2, tslice #3),
    #201 MultiTip(data #1),
    #300 Chart(data #1),
    #998 Empty(),
    #999 Incompatible(reason #1),
});

