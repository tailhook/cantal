use rotor_carbon;
use rotor::mio::tcp::TcpStream;

use stats::Stats;
use rotorloop::Context;

mod config;
mod util;
mod cgroups;
mod appmetrics;

pub use self::config::{Config, validator};
pub type Sender<'a> = rotor_carbon::Sender<'a, Context, TcpStream>;


pub fn send(snd: &mut Sender, cfg: &Config, stats: &Stats) {
    if cfg.enable_cgroup_stats {
        cgroups::scan(snd, cfg, stats);
    }
    if cfg.enable_application_metrics {
        appmetrics::scan(snd, cfg, stats);
    }
}
