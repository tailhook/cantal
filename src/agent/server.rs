use std::sync::RwLock;
use std::time::Duration;
use std::collections::{HashMap};
use serialize::json::Json;

use super::aio;
use super::scan;
use super::staticfiles;
use super::aio::http;
use super::stats::{Stats, Key};
use super::scan::processes::Pid;
use super::history::{Value};


const SHORT_HISTORY: usize = 30;


#[derive(Encodable)]
struct StatusData {
    pub startup_time: u64,
    pub scan_time: u64,
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

/*
#[derive(Encodable)]
struct DetailsData<'a> {
    pub startup_time: u64,
    pub scan_time: u64,
    pub machine: &'a scan::machine::MachineStats,
}
*/

#[derive(Encodable)]
struct ProcessesData<'a> {
    boot_time: Option<u64>,
    all: &'a Vec<scan::processes::MinimalProcess>,
}

/*
#[derive(Encodable)]
struct ProcessData<'a> {
    pub pid: Pid,
    pub process: &'a scan::processes::MinimalProcess,
    pub values: &'a Vec<(Json, Value)>,
}

#[derive(Encodable)]
struct ValuesData<'a> {
    pub items: Vec<ProcessData<'a>>,
}
*/


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
                scan_time: stats.scan_time,
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
            /*
            "/details.json" => Ok(http::reply_json(req, &DetailsData {
                startup_time: stats.startup_time,
                scan_time: stats.scan_time,
                machine: &stats.machine,
            })),
            "/values.json" => Ok(http::reply_json(req, &ValuesData {
                items: stats.processes.all.iter()
                    .filter_map(|prc| stats.processes.values.get(&prc.pid)
                        .map(|val| ProcessData {
                            pid: prc.pid,
                            process: prc,
                            values: val,
                            }))
                    .collect(),
            })),
            */
            _ => Err(http::Error::NotFound),
        }
    }
}


pub fn run_server<'x>(stats: &RwLock<Stats>, host: String, port: u16)
    -> Result<(), String>
{
    let handler: &for<'b> Fn(&'b aio::http::Request<'b>)
        -> Result<aio::http::Response, aio::http::Error>
        = &|&:req| {
        handle_request(stats, req)
    };
    let mut main = try!(aio::MainLoop::new()
        .map_err(|e| format!("Can't create main loop: {}", e)));
    try!(main.add_http_server(host.as_slice(), port, handler)
        .map_err(|e| format!("Can't bind {}:{}: {}", host, port, e)));
    main.run();
}
