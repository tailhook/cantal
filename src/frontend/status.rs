use std::sync::{Arc, RwLock};
use std::i32;
use std::time::{SystemTime, UNIX_EPOCH, Duration};

use self_meter_http::{Meter, ThreadReport, ProcessReport};
use juniper::{FieldError, ID};
use serde_json;

use frontend::graphql::ContextRef;
use frontend::quick_reply::{reply, respond};
use frontend::routing::Format;
use frontend::{Request};
use gossip::{NUM_PEERS, NUM_STALE};
use stats::Stats;
use storage::StorageStats;

#[derive(Serialize)]
pub struct StatusData<'a> {
    version: &'a str,
    startup_time: u64,
    scan_duration: u32,
    storage: StorageStats,
    boot_time: Option<u64>,
    self_report: ProcessReport<'a>,
    threads_report: ThreadReport<'a>,
    num_peers: i64,
    num_stale: i64,
}

pub struct GData<'a> {
    ctx: &'a ContextRef<'a>,
}

graphql_object!(<'a> GData<'a>: () as "Status" |&self| {
    description: "Status data for cantal itself"
    field startup_time() -> f64 {
        self.ctx.stats.startup_time as f64
    }
    field current_time() -> f64 {
        let n = SystemTime::now();
        return to_ms(n.duration_since(UNIX_EPOCH).expect("valid now")) as f64;
    }
    field scan_duration() -> i32 {
        self.ctx.stats.scan_duration as i32
    }
    field boot_time() -> Option<f64> {
        self.ctx.stats.boot_time.map(|t| t as f64)
    }
    field self_report() -> GProcessReport {
        GProcessReport(self.ctx.meter.clone())
    }
    field threads_report() -> GThreadsReport {
        GThreadsReport(self.ctx.meter.clone())
    }
    field processes() -> i32 {
        self.ctx.stats.processes.len() as i32
    }
    field fine_values() -> i32 {
        self.ctx.stats.history.fine.values.len() as i32
    }
    field tip_values() -> i32 {
        self.ctx.stats.history.tip.values.len() as i32
    }
    field id() -> ID {
        self.ctx.stats.id.to_hex().into()
    }
    field hostname() -> &String {
        &self.ctx.stats.hostname
    }
    field name() -> &String {
        &self.ctx.stats.name
    }
    field cluster_name() -> &Option<String> {
        &self.ctx.stats.cluster_name
    }
    field version() -> &'static str {
        env!("CARGO_PKG_VERSION")
    }
    field num_peers() -> i32 {
        NUM_PEERS.get() as i32
    }
    field num_stale() -> i32 {
        NUM_STALE.get() as i32
    }
    field has_remote() -> bool {
        self.ctx.remote.started()
    }
});

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
                version: env!("CARGO_PKG_VERSION"),
                startup_time: stats.startup_time,
                scan_duration: stats.scan_duration,
                storage: stats.storage,
                boot_time: stats.boot_time,
                self_report: meter.process_report(),
                threads_report: meter.thread_report(),
                num_peers: NUM_PEERS.get(),
                num_stale: NUM_STALE.get(),
            }
        ))
    })
}

pub fn graph<'a>(ctx: &'a ContextRef<'a>) -> Result<GData<'a>, FieldError>
{
    Ok(GData { ctx })
}

fn to_ms(dur: Duration) -> u64 {
    return dur.as_secs() * 1000 + dur.subsec_nanos() as u64 / 1000_000;
}
