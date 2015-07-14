use std::collections::HashMap;

use mio::{Token, Timeout, EventSet};

use super::deps::Dependencies;


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
    dependencies: Dependencies,
    connections: HashMap<Token, Cell>,
}
