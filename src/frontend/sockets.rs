use std::sync::{Arc, RwLock};

use stats::Stats;
use frontend::{Request};
use frontend::routing::Format;
use frontend::quick_reply::{reply, respond};


pub fn serve<S: 'static>(stats: &Arc<RwLock<Stats>>, format: Format)
    -> Request<S>
{
    let stats = stats.clone();
    reply(move |e| {
        let stats: &Stats = &*stats.read().expect("stats not poisoned");
        Box::new(respond(e, format, &stats.connections))
    })
}
