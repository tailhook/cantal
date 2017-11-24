use std::thread;
use std::process::exit;
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;

use abstract_ns::{HostResolve};
use ns_router::{Router, SubscribeExt, Config as NsConfig};
use futures::{Stream, Future};
use ns_std_threaded;
use self_meter::Meter;
use tk_carbon;
use tk_easyloop::{self, handle, spawn, interval};

use carbon;
use gossip;
use configs::Configs;
use stats::Stats;
use remote::Remote;
use storage::Storage;


quick_error! {
    #[derive(Debug)]
    pub enum InitError {
        Gossip(e: gossip::InitError) {
            from()
            display("error initializing gossip subsystem: {:?}", e)
        }
    }
}


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

        let mut keep_router = None;

        tk_easyloop::run_forever(|| -> Result<(), InitError> {

            let router = Router::from_config(&NsConfig::new()
                .set_fallthrough(ns_std_threaded::ThreadedResolver::new()
                    .null_service_resolver()
                    .interval_subscriber(Duration::new(1, 0), &handle()))
                .done(), &tk_easyloop::handle());
            keep_router = Some(router.clone());

            spawn_self_scan(meter);

            if let Some(gossip) = gossip.take() {
                gossip.spawn(&remote, &storage)?;
            }

            for cfg in &configs.carbon {
                let (carbon, init) = tk_carbon::Carbon::new(
                    &tk_carbon::Config::new().done());
                init.connect_to(
                    router.subscribe_many(&[&cfg.host], cfg.port),
                    &handle());
                let ivl = Duration::new(cfg.interval as u64, 0);
                let carbon = carbon.clone();
                let cfg = cfg.clone();
                let stats = stats.clone();
                spawn(interval(ivl)
                    .map_err(|_| -> () { unreachable!() })
                    .map(move |()| -> () {
                        debug!("Sending data to carbon {}:{}",
                            cfg.host, cfg.port);
                        carbon::send(&carbon, &cfg,
                                     &stats.read().expect("Can't lock stats"));
                    }).for_each(|()| Ok(())));
            }

            Ok(())
        }).map_err(|e| {
            error!("Error initializing tokio loop: {}", e);
            exit(1);
        }).expect("looping forever");
    });
}
