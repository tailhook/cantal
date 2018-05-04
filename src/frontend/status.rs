use std::sync::{Arc, RwLock};

use self_meter_http::{Meter, ThreadReport, ProcessReport};
use juniper::FieldError;

use storage::StorageStats;
use stats::Stats;
use frontend::{Request};
use frontend::routing::Format;
use frontend::quick_reply::{reply, respond};
use frontend::graphql::ContextRef;

#[derive(Serialize)]
pub struct StatusData<'a> {
    startup_time: u64,
    scan_duration: u32,
    storage: StorageStats,
    boot_time: Option<u64>,
    self_report: ProcessReport<'a>,
    threads_report: ThreadReport<'a>,
}

#[derive(Serialize, GraphQLObject)]
#[graphql(name="Status", description="Status data")]
pub struct GData {
    startup_time: f64,
    scan_duration: i32,
    //storage: StorageStats,
    boot_time: Option<f64>,
}

pub fn serve<S: 'static>(meter: &Meter, stats: &Arc<RwLock<Stats>>,
    format: Format)
    -> Request<S>
{
    let meter = meter.clone();
    let stats = stats.clone();
    reply(move |e| {
        let stats: &Stats = &*stats.read().expect("stats not poisoned");
        Box::new(respond(e, format,
            &StatusData {
                startup_time: stats.startup_time,
                scan_duration: stats.scan_duration,
                storage: stats.storage,
                boot_time: stats.boot_time,
                self_report: meter.process_report(),
                threads_report: meter.thread_report(),
            }
        ))
    })
}

pub fn graph(ctx: &ContextRef) -> Result<GData, FieldError>
{
    Ok(GData {
        startup_time: ctx.stats.startup_time as f64,
        scan_duration: ctx.stats.scan_duration as i32,
        //storage: ctx.stats.storage,
        boot_time: ctx.stats.boot_time.map(|x| x as f64),
        //self_report: ctx.meter.process_report(),
        //threads_report: ctx.meter.thread_report(),
    })
}
