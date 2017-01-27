use std::thread;
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;

use futures::{Stream, empty};
use self_meter::Meter;
use tk_easyloop;

use configs::Configs;
use stats::Stats;


fn spawn_self_scan(meter: Arc<Mutex<Meter>>) {
    tk_easyloop::handle().spawn(
        tk_easyloop::interval(Duration::new(1, 0)).map(move |()| {
            meter.lock().expect("meter is not poisoned")
            .scan()
            .map_err(|e| error!("Self-scan error: {}", e)).ok();
        }).or_else(|_| Ok(())).for_each(|()| Ok(())));
}


// All new async things should be in tokio main loop
pub fn start(_configs: &Configs, stats: &Arc<RwLock<Stats>>,
    meter: &Arc<Mutex<Meter>>)
{
    let meter = meter.clone();
    let _stats = stats.clone();
    debug!("Starting tokio loop");

    thread::spawn(move || {
        meter.lock().unwrap().track_current_thread("tokio");
        tk_easyloop::run(|| {
            spawn_self_scan(meter);
            empty::<(), ()>()
        })
    });
}
