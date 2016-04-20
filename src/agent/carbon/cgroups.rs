use std::collections::HashMap;

use cantal::Value::{Integer};
use history::Value;
use history::Backlog;
use history::CounterHistory;

use stats::Stats;

use super::config::Config;
use super::Sender;

#[derive(Debug)]
struct CGroup<'a> {
    name: &'a str,
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

fn get_rate_from_counter(hist: &CounterHistory, blog: &Backlog, num: usize)
    -> Option<(u64, u64)>
{
    hist.history(blog.age).enumerate().skip(1).take(num)
    .filter_map(|(idx, x)| x.map(|y| (idx, y))).last()
    .map(|(idx, x)| {
        let cur = (blog.age - hist.age()) as usize;
        assert!(idx >= cur);
        (hist.tip().saturating_sub(x),
            (blog.timestamps[cur].0 - blog.timestamps[idx].0))
    })
}

fn get_counter_diff(val: &Value, blog: &Backlog, num: usize)
    -> Option<(u64, u64)>
{
    use history::Value::Counter;
    match val {
        &Counter(ref hist) => {
            get_rate_from_counter(hist, blog, num)
        }
        _ => None,
    }
}

fn graphite_data(val: &Value, blog: &Backlog, num: usize)
    -> Option<f64>
{
    use history::Value::{Counter, Integer};
    match val {
        &Counter(ref hist) => {
            get_rate_from_counter(hist, blog, num)
            .map(|(value, millis)| {
                (value as f64)*1000.0/(millis as f64)
            })
        }
        &Integer(ref hist) => {
            Some(hist.tip() as f64)
        }
        _ => None,
    }
}

pub fn scan(sender: &mut Sender, cfg: &Config, stats: &Stats)
{
    let ref backlog = stats.history.fine;
    if backlog.timestamps.len() < 2 {
        return;
    }
    let mut cgroups = Vec::with_capacity(stats.cgroups.len());
    let mut pids = HashMap::with_capacity(stats.processes.len());
    for (name, cgroup) in &stats.cgroups {
        let idx = cgroups.len();
        cgroups.push(CGroup::new(name));
        for pid in &cgroup.pids {
            pids.insert(*pid, idx);
        }
    }
    let timestamp = backlog.timestamps[0].0;
    let cut = timestamp - (cfg.interval as u64)*1000;
    let num = backlog.timestamps.iter().enumerate()
        .skip_while(|&(_, &(x, d))| x-d as u64 > cut)
        .next().map(|(x, _)| x)
        .unwrap_or(backlog.timestamps.len() - 1);
    let unixtime = timestamp / 1000;
    for (key, value) in backlog.values.iter() {
        key.get_with("pid", |pid_str| {
            let pid = pid_str.parse().ok();
            if let Some(&grp_num) = pid.and_then(|x| pids.get(&x)) {
                let ref mut grp = cgroups[grp_num];
                key.get_with("metric", |metric| {
                    match metric {
                        "vsize" => {
                            if let Integer(val) = value.tip_value() {
                                grp.vsize += val as u64;
                            }
                        }
                        "rss" => {
                            if let Integer(val) = value.tip_value() {
                                grp.rss += val as u64;
                            }
                        }
                        "num_threads" => {
                            if let Integer(val) = value.tip_value() {
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
                });
                // TODO(tailhook) get metrics with "group" but not state
                key.get_with("state", |statename| {
                    key.get_with("metric", |metric| {
                        if let Some(v) = graphite_data(value, backlog, num) {
                            let usermet = format!("states.{}.{}",
                                statename, metric);
                            let valptr = grp.user_metrics.entry(usermet)
                                         .or_insert(0.0);
                            *valptr += v;
                        }
                    });
                });
                key.get_with("group", |group| {
                    key.get_with("metric", |metric| {
                        if let Some(v) = graphite_data(value, backlog, num) {
                            let usermet = format!("groups.{}.{}",
                                group, metric);
                            let valptr = grp.user_metrics.entry(usermet)
                                         .or_insert(0.0);
                            *valptr += v;
                        }
                    });
                });
            }
        });
    }
    let cls = stats.cluster_name.as_ref().map(|x| &x[..])
              .unwrap_or("no-cluster");
    for cgroup in cgroups {
        sender.add_value_at(
            format_args!("cantal.{}.{}.cgroups.{}.vsize",
                cls, stats.hostname, cgroup.name),
            cgroup.vsize, unixtime);
        sender.add_value_at(
            format_args!("cantal.{}.{}.cgroups.{}.rss",
                cls, stats.hostname, cgroup.name),
            cgroup.rss, unixtime);
        sender.add_value_at(
            format_args!("cantal.{}.{}.cgroups.{}.num_processes",
                cls, stats.hostname, cgroup.name),
            cgroup.num_processes, unixtime);
        sender.add_value_at(
            format_args!("cantal.{}.{}.cgroups.{}.num_threads",
                cls, stats.hostname, cgroup.name),
            cgroup.num_threads, unixtime);
        sender.add_value_at(
            format_args!("cantal.{}.{}.cgroups.{}.user_cpu_percent",
                cls, stats.hostname, cgroup.name),
            cgroup.user_cpu, unixtime);
        sender.add_value_at(
            format_args!("cantal.{}.{}.cgroups.{}.system_cpu_percent",
                cls, stats.hostname, cgroup.name),
            cgroup.system_cpu, unixtime);
        sender.add_value_at(
            format_args!("cantal.{}.{}.cgroups.{}.read_bps",
                cls, stats.hostname, cgroup.name),
            cgroup.read_bytes, unixtime);
        sender.add_value_at(
            format_args!("cantal.{}.{}.cgroups.{}.write_bps",
                cls, stats.hostname, cgroup.name),
            cgroup.write_bytes, unixtime);
        for (key, value) in cgroup.user_metrics {
            sender.add_value_at(
                format_args!("cantal.{}.{}.cgroups.{}.{}",
                    cls, stats.hostname, cgroup.name, key),
                value, unixtime);
        }
    }
}

impl<'a> CGroup<'a> {
    fn new(name: &str) -> CGroup {
        CGroup {
            name: name,
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
