use std::collections::HashMap;

use history::{Key, SnapTime, Chunk};
use values::Value;


pub enum Dataset {
    MultiSeries(Vec<(Key, Chunk)>),
    SingleSeries(Chunk),
    MultiTip(Vec<(Key, Value)>),
    SingleTip(Vec<(Key, Value)>),
    Chart(HashMap<String, usize>),
    // TODO(tailhook) multi-chart
}
