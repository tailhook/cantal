use std::sync::{RwLock, Arc};
use std::time::Duration;

use tk_carbon::{Carbon, Config as CarbonConfig};
use tk_easyloop::{spawn, interval, handle};

use ns_env_config::Router;
use futures::{Stream};
use failure::Error;
use stats::Stats;
use configs::Configs;

mod config;
mod util;
mod cgroups;
mod appmetrics;
mod myself;


pub use self::config::{Config, validator};


fn send(snd: &Carbon, cfg: &Config, stats: &Stats) {
    if cfg.enable_cgroup_stats {
        cgroups::scan(snd, cfg, stats);
    }
    if cfg.enable_application_metrics {
        appmetrics::scan(snd, cfg, stats);
        // consider this a special case of application metrics
        myself::scan(snd, cfg, stats);
    }
}

pub fn spawn_sinks(ns: &Router,
    configs: &Arc<Configs>, stats: &Arc<RwLock<Stats>>)
    -> Result<(), Error>
{
    for cfg in &configs.carbon {
        let (carbon, init) = Carbon::new(&CarbonConfig::new().done());
        init.connect_to(ns.subscribe_many(&[&cfg.host], cfg.port), &handle());
        let ivl = Duration::new(cfg.interval as u64, 0);
        let carbon = carbon.clone();
        let cfg = cfg.clone();
        let stats = stats.clone();
        spawn(interval(ivl)
            .map_err(|_| -> () { unreachable!() })
            .map(move |()| -> () {
                debug!("Sending data to carbon {}:{}",
                    cfg.host, cfg.port);
                send(&carbon, &cfg, &stats.read().expect("Can't lock stats"));
            }).for_each(|()| Ok(())));
    }
    Ok(())
}
