use std::str::FromStr;
use std::str::from_utf8;
use std::sync::RwLock;
use rustc_serialize::json::Json;

use super::aio;
use super::scan;
use super::staticfiles;
use super::aio::http;
use super::util::tree_collect;
use super::stats::{Stats, Key};
use super::scan::processes::Pid;


const SHORT_HISTORY: usize = 30;


#[derive(RustcEncodable)]
struct StatusData {
    pub startup_time: u64,
    pub scan_duration: u32,
    pub store_time: u64,
    pub store_timestamp: u64,
    pub store_duration: u32,
    pub store_size: usize,
    pub boot_time: Option<u64>,

    pub load_avg_1min: Json,
    pub load_avg_5min: Json,
    pub load_avg_15min: Json,
    pub mem_total: Json,
    pub mem_free: Json,
    pub mem_cached: Json,
    pub mem_buffers: Json,

    pub history_timestamps: Vec<(u64, u32)>,
    pub cpu_user: Json,
    pub cpu_nice: Json,
    pub cpu_system: Json,
    pub cpu_idle: Json,

}

#[derive(RustcEncodable)]
struct Metrics {
    pub latest: Vec<(Json, Json)>,
    pub history: Vec<(Json, Json)>,
    pub history_timestamps: Vec<(u64, u32)>,
}

#[derive(RustcEncodable)]
struct ProcessesData<'a> {
    boot_time: Option<u64>,
    all: &'a Vec<scan::processes::MinimalProcess>,
}

#[derive(RustcEncodable)]
struct ProcessData<'a> {
    pub pid: Pid,
    pub process: &'a scan::processes::MinimalProcess,
    pub values: Vec<(Json, Json)>,
}

#[derive(RustcEncodable)]
struct ProcessValues<'a> {
    pub processes: Vec<ProcessData<'a>>,
}

fn process_values<'x>(stats: &'x Stats) -> Vec<ProcessData<'x>> {
    let mut tree = tree_collect(stats.history
        .latest(|key| key.get("pid").is_some())
        .into_iter().map(|(key, val)| {
            let pid = FromStr::from_str(
                key["pid"].as_string().unwrap_or("0")).unwrap_or(0);
            (pid, (key, val))
        }));
    stats.processes.iter().filter_map(|p| {
        tree.remove(&p.pid).map(|val| ProcessData {
            pid: p.pid,
            process: p,
            values: val,
        })
    }).collect()
}


fn handle_request(stats: &RwLock<Stats>, req: &http::Request)
    -> Result<http::Response, http::Error>
{
    if  req.uri().starts_with("/js") ||
        req.uri().starts_with("/css/") ||
        req.uri().starts_with("/fonts/") ||
        req.uri() == "/"
    {
        return staticfiles::serve(req);
    } else {
        let stats = stats.read().unwrap();
        let ref h = stats.history;
        match req.uri() {
            "/status.json" => Ok(http::reply_json(req, &StatusData {
                startup_time: stats.startup_time,
                scan_duration: stats.scan_duration,
                store_time: stats.store_time,
                store_duration: stats.store_duration,
                store_timestamp: stats.store_timestamp,
                store_size: stats.store_size,
                boot_time: stats.boot_time,

                load_avg_1min: h.get_tip_json(&Key::metric("load_avg_1min")),
                load_avg_5min: h.get_tip_json(&Key::metric("load_avg_5min")),
                load_avg_15min: h.get_tip_json(&Key::metric("load_avg_15min")),
                mem_total: h.get_tip_json(&Key::metric("memory.MemTotal")),
                mem_free: h.get_tip_json(&Key::metric("memory.MemFree")),
                mem_buffers: h.get_tip_json(&Key::metric("memory.Buffers")),
                mem_cached: h.get_tip_json(&Key::metric("memory.Cached")),


                history_timestamps: h.get_timestamps(SHORT_HISTORY),
                cpu_user: h.get_history_json(
                    &Key::metric("cpu.user"), SHORT_HISTORY),
                cpu_nice: h.get_history_json(
                    &Key::metric("cpu.nice"), SHORT_HISTORY),
                cpu_system: h.get_history_json(
                    &Key::metric("cpu.system"), SHORT_HISTORY),
                cpu_idle: h.get_history_json(
                    &Key::metric("cpu.idle"), SHORT_HISTORY),
            })),
            "/all_processes.json" => Ok(http::reply_json(req, &ProcessesData {
                boot_time: stats.boot_time,
                all: &stats.processes,
            })),
            "/details.json" => Ok(http::reply_json(req, &Metrics {
                history_timestamps: h.get_timestamps(SHORT_HISTORY),
                latest: stats.history.latest(|key| {
                    key.get("metric")
                    .map(|x| x.starts_with("memory."))
                    .unwrap_or(false)
                }),
                history: stats.history.history(SHORT_HISTORY, |key| {
                    key.get("metric")
                    .map(|x| x.starts_with("net.") || x.starts_with("disk."))
                    .unwrap_or(false)
                }),
            })),
            "/process_values.json" => Ok(http::reply_json(req, &ProcessValues {
                processes: process_values(&*stats),
            })),
            "/states.json" => Ok(http::reply_json(req, &Metrics {
                history_timestamps: vec!(),
                history: vec!(),
                latest: stats.history.latest(|key| {
                    key.get("state").is_some()
                }),
            })),
            "/query.json" => Ok(http::reply_json(req,
                &vec!(req.body.map(|x| from_utf8(x).unwrap())))),
            _ => Err(http::Error::NotFound),
        }
    }
}


pub fn run_server(stats: &RwLock<Stats>, host: String, port: u16)
    -> Result<(), String>
{
    let handler: &for<'b> Fn(&'b aio::http::Request<'b>)
        -> Result<aio::http::Response, aio::http::Error>
        = &|req| {
        handle_request(stats, req)
    };
    let mut main = try!(aio::MainLoop::new()
        .map_err(|e| format!("Can't create main loop: {}", e)));
    try!(main.add_http_server(&host, port, handler)
        .map_err(|e| format!("Can't bind {}:{}: {}", host, port, e)));
    main.run();
}
