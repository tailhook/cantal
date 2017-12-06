use std::thread;
use std::process::exit;
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;

use failure::Error;
use futures::{Stream, Future};
use ns_env_config;
use self_meter::Meter;
use tk_easyloop::{self, handle, spawn, interval};

use carbon;
use gossip;
use configs::Configs;
use stats::Stats;
use remote::Remote;
use storage::Storage;


fn spawn_self_scan(meter: Arc<Mutex<Meter>>) {
    spawn(
        interval(Duration::new(1, 0)).for_each(move |()| {
            meter.lock().expect("meter is not poisoned")
            .scan()
            .map_err(|e| error!("Self-scan error: {}", e)).ok();
            Ok(())
        }).map_err(|_| -> () { unreachable!() }));
}


// All new async things should be in tokio main loop
pub fn start(mut gossip: Option<gossip::GossipInit>,
    configs: &Arc<Configs>, stats: &Arc<RwLock<Stats>>,
    meter: &Arc<Mutex<Meter>>, remote: &Remote, storage: &Arc<Storage>)
{
    let meter = meter.clone();
    let remote = remote.clone();
    let storage = storage.clone();
    let stats = stats.clone();
    let configs = configs.clone();
    debug!("Starting tokio loop");


    thread::spawn(move || {
        meter.lock().unwrap().track_current_thread("tokio");

        tk_easyloop::run_forever(|| -> Result<(), Error> {
            let router = ns_env_config::init(&handle())?;

            spawn_self_scan(meter);

            if let Some(gossip) = gossip.take() {
                gossip.spawn(&remote, &storage)?;
            }

            carbon::spawn_sinks(&router, &configs, &stats)?;

            Ok(())
        }).map_err(|e| {
            error!("Error initializing tokio loop: {}", e);
            exit(1);
        }).expect("looping forever");
    });
}
