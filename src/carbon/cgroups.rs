use std::collections::HashMap;
use std::time::{Duration, UNIX_EPOCH};

use cantal::Value::{Integer};
use regex::Regex;
use tk_carbon::Carbon;

use stats::Stats;

use super::config::Config;
use super::util::{graphite_data, get_counter_diff};


lazy_static! {
    /// A cgroup name created by lithos_cmd looks like:
    ///
    ///  lithos.role-name:cmd.task-name.31012
    ///
    /// Since carbon doesn't like too many metrics we need to cut the last
    /// part of the group name, which is a process pid (i.e. basically a
    /// new value each time)
    static ref LITHOS_CMD: Regex = Regex::new(
        r#"^(lithos\.[^:]+:cmd\.[^\.]+)\.\d+$"#)
        .expect("LITHOST_CMD regex compiles");
}


#[derive(Debug)]
struct CGroup {
    vsize: u64,
    rss: u64,
    num_threads: u64,
    num_processes: u64,
    user_cpu: f64,
    system_cpu: f64,
    read_bytes: f64,
    write_bytes: f64,
    user_metrics: HashMap<String, f64>,
}

pub fn scan(sender: &Carbon, cfg: &Config, stats: &Stats)
{
    let ref backlog = stats.history.fine;
    if backlog.timestamps.len() < 2 {
        return;
    }
    let mut cgroups = HashMap::<String, CGroup>::new();
    let timestamp = backlog.timestamps[0].0;
    let cut = timestamp - (cfg.interval as u64)*1000;
    let num = backlog.timestamps.iter().enumerate()
        .skip_while(|&(_, &(x, d))| x-d as u64 > cut)
        .next().map(|(x, _)| x)
        .unwrap_or(backlog.timestamps.len() - 1);
    let cut_age = backlog.age.saturating_sub(num as u64);
    let unixtime = UNIX_EPOCH + Duration::from_millis(timestamp);
    for (key, value) in backlog.values.iter() {
        key.get_with("cgroup", |cgroup| {
            // simplify lithos_cmd's groups (see description of LITHOS_CMD)
            let captures = LITHOS_CMD.captures(cgroup);
            let cgroup = match captures {
                Some(ref capt) => capt.get(1).unwrap().as_str(),
                None => cgroup,
            };
            key.get_with("metric", |metric| {
                let grp = cgroups.entry(cgroup.to_owned())
                    .or_insert_with(CGroup::new);
                match metric {
                    "vsize" => {
                        if let Some(Integer(val)) = value.tip_or_none(cut_age)
                        {
                            grp.vsize += val as u64;
                        }
                    }
                    "rss" => {
                        if let Some(Integer(val)) = value.tip_or_none(cut_age)
                        {
                            grp.rss += val as u64;
                        }
                    }
                    "num_threads" => {
                        if let Some(Integer(val)) = value.tip_or_none(cut_age)
                        {
                            grp.num_threads += val as u64;
                            grp.num_processes += 1;
                        }
                    }
                    "user_time" => {
                        let diff = get_counter_diff(value, backlog, num);
                        if let Some((value, millis)) = diff {
                            // TODO(tailhook) div by cpu ticks not millis
                            let mfloat = millis as f64;
                            grp.user_cpu += (value as f64)*1000.0/mfloat;
                        }
                    }
                    "system_time" => {
                        let diff = get_counter_diff(value, backlog, num);
                        if let Some((value, millis)) = diff {
                            // TODO(tailhook) div by cpu ticks not millis
                            let mfloat = millis as f64;
                            grp.system_cpu += (value as f64)*1000.0/mfloat;
                        }
                    }
                    "read_bytes" => {
                        let diff = get_counter_diff(value, backlog, num);
                        if let Some((value, millis)) = diff {
                            let mfloat = millis as f64;
                            grp.read_bytes += (value as f64)*1000.0/mfloat;
                        }
                    }
                    "write_bytes" => {
                        let diff = get_counter_diff(value, backlog, num);
                        if let Some((value, millis)) = diff {
                            let mfloat = millis as f64;
                            grp.write_bytes += (value as f64)*1000.0/mfloat;
                        }
                    }
                    _ => {}
                }
                key.get_with("state", |statename| {
                    if let Some(v) = graphite_data(value, backlog, num) {
                        let usermet = format!("states.{}.{}",
                            statename, metric);
                        let valptr = grp.user_metrics.entry(usermet)
                                     .or_insert(0.0);
                        *valptr += v;
                    }
                });
                key.get_with("group", |group| {
                    if let Some(v) = graphite_data(value, backlog, num) {
                        let usermet = format!("groups.{}.{}",
                            group, metric);
                        let valptr = grp.user_metrics.entry(usermet)
                                     .or_insert(0.0);
                        *valptr += v;
                    }
                });
            });
        });
    }
    let cls = stats.cluster_name.as_ref().map(|x| &x[..])
              .unwrap_or("no-cluster");
    for (name, cgroup) in cgroups {
        sender.add_value_at(
            format_args!("cantal.{}.{}.cgroups.{}.vsize",
                cls, stats.hostname, name),
            cgroup.vsize, unixtime);
        sender.add_value_at(
            format_args!("cantal.{}.{}.cgroups.{}.rss",
                cls, stats.hostname, name),
            cgroup.rss, unixtime);
        sender.add_value_at(
            format_args!("cantal.{}.{}.cgroups.{}.num_processes",
                cls, stats.hostname, name),
            cgroup.num_processes, unixtime);
        sender.add_value_at(
            format_args!("cantal.{}.{}.cgroups.{}.num_threads",
                cls, stats.hostname, name),
            cgroup.num_threads, unixtime);
        sender.add_value_at(
            format_args!("cantal.{}.{}.cgroups.{}.user_cpu_percent",
                cls, stats.hostname, name),
            cgroup.user_cpu, unixtime);
        sender.add_value_at(
            format_args!("cantal.{}.{}.cgroups.{}.system_cpu_percent",
                cls, stats.hostname, name),
            cgroup.system_cpu, unixtime);
        sender.add_value_at(
            format_args!("cantal.{}.{}.cgroups.{}.read_bps",
                cls, stats.hostname, name),
            cgroup.read_bytes, unixtime);
        sender.add_value_at(
            format_args!("cantal.{}.{}.cgroups.{}.write_bps",
                cls, stats.hostname, name),
            cgroup.write_bytes, unixtime);
        for (key, value) in cgroup.user_metrics {
            sender.add_value_at(
                format_args!("cantal.{}.{}.cgroups.{}.{}",
                    cls, stats.hostname, name, key),
                value, unixtime);
        }
    }
}

impl CGroup {
    fn new() -> CGroup {
        CGroup {
            vsize: 0,
            rss: 0,
            num_threads: 0,
            num_processes: 0,
            user_cpu: 0.,
            system_cpu: 0.,
            read_bytes: 0.,
            write_bytes: 0.,
            user_metrics: HashMap::new(),
        }
    }
}
