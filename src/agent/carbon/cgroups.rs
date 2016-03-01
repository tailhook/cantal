use std::collections::HashMap;

use cantal::Value::{Integer};
use history::Value::{self, Counter};
use history::Backlog;

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
}

fn get_counter_diff(val: &Value, blog: &Backlog, num: usize)
    -> Option<(u64, u64)>
{
    if let &Counter(ref hist) = val {
        hist.history(blog.age).enumerate().skip(1).take(num)
         .filter_map(|(idx, x)| x.map(|y| (idx, y))).last()
         .map(|(idx, x)| {
            let cur = (blog.age - hist.age()) as usize;
            assert!(idx >= cur);
            (hist.tip().saturating_sub(x),
                (blog.timestamps[cur].0 - blog.timestamps[idx].0))
        })
    } else {
        None
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
            }
        });
    }
    for cgroup in cgroups {
        sender.add_value_at(
            format_args!("cantal.{}.cgroups.{}.vsize",
                stats.hostname, cgroup.name),
            cgroup.vsize, unixtime);
        sender.add_value_at(
            format_args!("cantal.{}.cgroups.{}.rss",
                stats.hostname, cgroup.name),
            cgroup.rss, unixtime);
        sender.add_value_at(
            format_args!("cantal.{}.cgroups.{}.num_processes",
                stats.hostname, cgroup.name),
            cgroup.num_processes, unixtime);
        sender.add_value_at(
            format_args!("cantal.{}.cgroups.{}.num_threads",
                stats.hostname, cgroup.name),
            cgroup.num_threads, unixtime);
        sender.add_value_at(
            format_args!("cantal.{}.cgroups.{}.user_cpu_percent",
                stats.hostname, cgroup.name),
            cgroup.user_cpu, unixtime);
        sender.add_value_at(
            format_args!("cantal.{}.cgroups.{}.system_cpu_percent",
                stats.hostname, cgroup.name),
            cgroup.system_cpu, unixtime);
        sender.add_value_at(
            format_args!("cantal.{}.cgroups.{}.read_bps",
                stats.hostname, cgroup.name),
            cgroup.read_bytes, unixtime);
        sender.add_value_at(
            format_args!("cantal.{}.cgroups.{}.write_bps",
                stats.hostname, cgroup.name),
            cgroup.write_bytes, unixtime);
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
        }
    }
}
