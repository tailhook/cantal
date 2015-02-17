pub struct Stats {
    pub startup_time: u64,
}

impl Stats {
    pub fn new() -> Stats {
        return Stats {
            startup_time: 7,
        };
    }
}
