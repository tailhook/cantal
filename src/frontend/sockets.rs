use std::sync::{Arc, RwLock};

use stats::Stats;
use frontend::{Request};
use frontend::routing::Format;
use frontend::quick_reply::{reply, respond};
use scan::processes::MinimalProcess;

#[derive(Serialize)]
struct ProcessesData<'a> {
    boot_time: Option<u64>,
    all: &'a Vec<MinimalProcess>,
}

pub fn serve<S: 'static>(stats: &Arc<RwLock<Stats>>, format: Format)
    -> Request<S>
{
    let stats = stats.clone();
    reply(move |e| {
        let stats: &Stats = &*stats.read().expect("stats not poisoned");
        Box::new(respond(e, format, &stats.connections))
    })
}
