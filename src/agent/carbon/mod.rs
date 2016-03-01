use rotor_carbon;
use rotor::mio::tcp::TcpStream;

use stats::Stats;
use rotorloop::Context;

mod config;
mod cgroups;

pub use self::config::{Config, validator};
pub type Sender<'a> = rotor_carbon::Sender<'a, Context, TcpStream>;


pub fn send(snd: &mut Sender, cfg: &Config, stats: &Stats) {
    if cfg.enable_cgroup_stats {
        cgroups::scan(snd, cfg, stats);
    }
}
