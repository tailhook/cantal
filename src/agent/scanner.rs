use std::sync::{Arc, RwLock};
use std::io::timer::sleep;
use std::time::duration::Duration;

use super::stats::Stats;
use super::scan::machine;


pub fn scan_loop(stats: Arc<RwLock<Stats>>) {
    loop {
        let value = machine::read();
        stats.write().unwrap().machine = value;
        sleep(Duration::seconds(2));
    }
}
