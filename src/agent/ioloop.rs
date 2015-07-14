use std::any::Any;
use std::collections::HashMap;

use mio::{Token, Timeout, EventSet};
use anymap::any::CloneAny;
use anymap::Map;

trait Consumer {
    fn ready(self) -> Option<(EventSet, Option<u64>, Self)>;
    fn timeout(self) -> Option<Self>;
}


struct Cell {
    event_set: EventSet,
    timeout: Option<(u64, Timeout)>,
    consumer: Box<Consumer>,
}


struct Handler {
    dependencies: Map<CloneAny+Sync+Send>,
    connections: HashMap<Token, Cell>,
}
