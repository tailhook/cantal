use self_meter_http::{Meter, ThreadReport, ProcessReport};

use frontend::{Request};
use frontend::routing::Format;
use frontend::quick_reply::{reply, respond};


pub fn serve<S: 'static>(meter: &Meter, //stats: RwLock<Stats>,
    format: Format)
    -> Request<S>
{
    #[derive(Serialize)]
    struct StatusData<'a> {
        // startup_time: u64,
        // scan_duration: u32,
        // storage: StorageStats,
        // boot_time: Option<u64>,
        self_report: ProcessReport<'a>,
        threads_report: ThreadReport<'a>,
    }
    let meter = meter.clone();
    reply(move |e| {
        //let stats: &Stats = &*stats.read();
        Box::new(respond(e, format,
            &StatusData {
                // startup_time: stats.startup_time,
                // scan_duration: stats.scan_duration,
                // storage: stats.storage,
                // boot_time: stats.boot_time,
                self_report: meter.process_report(),
                threads_report: meter.thread_report(),
            }
        ))
    })
}
