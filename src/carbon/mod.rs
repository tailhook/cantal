use tk_carbon::Carbon;

use stats::Stats;

mod config;
mod util;
mod cgroups;
mod appmetrics;


pub use self::config::{Config, validator};


pub fn send(snd: &Carbon, cfg: &Config, stats: &Stats) {
    if cfg.enable_cgroup_stats {
        cgroups::scan(snd, cfg, stats);
    }
    if cfg.enable_application_metrics {
        appmetrics::scan(snd, cfg, stats);
    }
}
