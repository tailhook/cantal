use rotor_carbon::Sender;

use stats::Stats;

mod config;
mod cgroups;

pub use self::config::{Config, validator};


pub fn send(snd: &mut Sender, cfg: &Config, stats: &Stats) {
    if cfg.enable_cgroup_stats {
        cgroups::scan(snd, cfg, stats);
    }
}
