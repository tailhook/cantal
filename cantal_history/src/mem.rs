
use {Tip, Backlog};

#[derive(Debug)]
pub struct History {
    // Values that are kept as fine-grained as possible (2-second interval)
    pub fine: Backlog,
    pub tip: Tip,
}

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

