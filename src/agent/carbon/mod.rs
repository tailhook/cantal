use rotor_carbon::Sender;

use super::stats::Stats;

mod config;

pub use self::config::{Config, validator};

pub fn send(snd: Sender, cfg: &Config, stats: &Stats) {
    unimplemented!();
}
