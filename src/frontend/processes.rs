use std::sync::{Arc, RwLock};
use std::time::{SystemTime, Duration, UNIX_EPOCH};

use stats::Stats;
use frontend::{Request};
use frontend::routing::Format;
use frontend::quick_reply::{reply, respond};
use frontend::graphql::ContextRef;
pub use scan::processes::MinimalProcess as Process;


// ------------------ old endpoint ---------------------

#[derive(Serialize)]
struct ProcessesData<'a> {
    boot_time: Option<u64>,
    all: &'a Vec<Process>,
}

pub fn serve<S: 'static>(stats: &Arc<RwLock<Stats>>, format: Format)
    -> Request<S>
{
    let stats = stats.clone();
    reply(move |e| {
        let stats: &Stats = &*stats.read().expect("stats not poisoned");
        Box::new(respond(e, format,
            &ProcessesData {
                boot_time: stats.boot_time,
                all: &stats.processes,
            }
        ))
    })
}

// ---------------------- graphql ----------------------

#[derive(GraphQLInputObject)]
#[graphql(name="ProcessFilter", description="Filter for processes")]
pub struct Filter {
    maximum_uptime: Option<i32>,
}

pub fn processes<'x>(ctx: &ContextRef<'x>, filter: Option<Filter>)
    -> Vec<&'x Process>
{
    let timestamp = filter.as_ref()
        .and_then(|x| x.maximum_uptime)
        .and_then(|x| {
            let dur = (SystemTime::now() - Duration::from_millis(x as u64))
                       .duration_since(UNIX_EPOCH).ok()?;
            dur.as_secs().checked_mul(1000)?
            .checked_add(dur.subsec_nanos() as u64 / 1000000)
        });
    return ctx.stats.processes.iter()
        .filter(|p| {
            timestamp.map(|ts| ts <= p.start_timestamp).unwrap_or(true)
        })
        .collect();
}
