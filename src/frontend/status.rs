use std::sync::{Arc, RwLock};
use std::i32;

use self_meter_http::{Meter, ThreadReport, ProcessReport};
use juniper::FieldError;
use serde_json;

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

#[derive(GraphQLObject)]
#[graphql(name="Status", description="Status data")]
pub struct GData {
    startup_time: f64,
    scan_duration: i32,
    //storage: StorageStats,
    boot_time: Option<f64>,
    self_report: GProcessReport,
    threads_report: GThreadsReport,
}

pub struct GProcessReport(Meter);
pub struct GThreadsReport(Meter);


// TODO(tailhook) rather make a serializer
fn convert(val: serde_json::Value) -> ::juniper::Value {
    use serde_json::Value as I;
    use juniper::Value as O;
    match val {
        I::Null => O::Null,
        I::Number(n) => {
            if let Some(i) = n.as_i64() {
                if i <= i32::MAX as i64 && i >= i32::MIN as i64 {
                    O::Int(i as i32)
                } else {
                    O::Float(i as f64)
                }
            } else {
                O::Float(n.as_f64().expect("can alwasy be float"))
            }
        }
        I::String(s) => O::String(s),
        I::Bool(v) => O::Boolean(v),
        I::Array(items) => O::List(items.into_iter().map(convert).collect()),
        I::Object(map) => {
            O::Object(map.into_iter().map(|(k, v)| (k, convert(v))).collect())
        }
    }
}

graphql_scalar!(GProcessReport as "ProcessReport" {
    description: "process perfromance information"
    resolve(&self) -> Value {
        convert(serde_json::to_value(self.0.process_report())
            .expect("serialize ProcessReport"))
    }
    from_input_value(_val: &InputValue) -> Option<GProcessReport> {
        unimplemented!();
    }
});

graphql_scalar!(GThreadsReport as "ThreadsReport" {
    description: "per-thread performance information"
    resolve(&self) -> Value {
        convert(serde_json::to_value(self.0.thread_report())
            .expect("serialize ThreadReport"))
    }
    from_input_value(_val: &InputValue) -> Option<GThreadsReport> {
        unimplemented!();
    }
});

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
        self_report: GProcessReport(ctx.meter.clone()),
        threads_report: GThreadsReport(ctx.meter.clone()),
    })
}
