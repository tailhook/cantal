use stats::Stats;
use tk_carbon::Carbon;

use super::config::Config;
use gossip::{NUM_PEERS, NUM_STALE};


pub fn scan(sender: &Carbon, _cfg: &Config, stats: &Stats) {
    let cls = stats.cluster_name.as_ref().map(|x| &x[..])
              .unwrap_or("no-cluster");
    sender.add_value(
        format_args!("cantal.{}.{}.apps.{}.groups.gossip.num_peers",
            cls, stats.hostname, "cantal"),
        NUM_PEERS.get());
    sender.add_value(
        format_args!("cantal.{}.{}.apps.{}.groups.gossip.num_stale",
            cls, stats.hostname, "cantal"),
        NUM_STALE.get());
}
