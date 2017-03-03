use std::time::{Duration, UNIX_EPOCH};

use stats::Stats;
use tk_carbon::Carbon;

use super::config::Config;
use super::util::{graphite_data};


pub fn scan(sender: &Carbon, cfg: &Config, stats: &Stats)
{
    let ref backlog = stats.history.fine;
    if backlog.timestamps.len() < 2 {
        return;
    }
    let timestamp = backlog.timestamps[0].0;
    let cut = timestamp - (cfg.interval as u64)*1000;
    let num = backlog.timestamps.iter().enumerate()
        .skip_while(|&(_, &(x, d))| x-d as u64 > cut)
        .next().map(|(x, _)| x)
        .unwrap_or(backlog.timestamps.len() - 1);
    let unixtime = UNIX_EPOCH + Duration::from_millis(timestamp);
    let cls = stats.cluster_name.as_ref().map(|x| &x[..])
              .unwrap_or("no-cluster");
    for (key, value) in backlog.values.iter() {
        key.get_with("appname", |app| {
            key.get_with("metric", |metric| {
                key.get_with("state", |statename| {
                    if let Some(v) = graphite_data(value, backlog, num) {
                        sender.add_value_at(
                            format_args!("cantal.{}.{}.apps.{}.states.{}.{}",
                                cls, stats.hostname, app, statename, metric),
                            v, unixtime);
                    }
                });
                key.get_with("group", |group| {
                    if let Some(v) = graphite_data(value, backlog, num) {
                        sender.add_value_at(
                            format_args!("cantal.{}.{}.apps.{}.groups.{}.{}",
                                cls, stats.hostname, app, group, metric),
                            v, unixtime);
                    }
                });
            });
        });
    }
}
