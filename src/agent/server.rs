use std::sync::RwLock;
use std::time::Duration;
use serialize::json::Json;

use super::aio;
use super::scan;
use super::staticfiles;
use super::aio::http;
use super::stats::Stats;


#[derive(Encodable)]
struct StatusData {
    pub startup_time: u64,
    pub scan_time: u64,

    pub load_avg_1min: Option<f32>,
    pub load_avg_5min: Option<f32>,
    pub load_avg_15min: Option<f32>,
    pub boot_time: Option<u64>,
}

#[derive(Encodable)]
struct DetailsData<'a> {
    pub startup_time: u64,
    pub scan_time: u64,
    pub machine: &'a scan::machine::MachineStats,
}

#[derive(Encodable)]
struct ProcessData<'a> {
    boot_time: Option<u64>,
    all: &'a Vec<scan::processes::MinimalProcess>,
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
        match req.uri() {
            "/status.json" => Ok(http::reply_json(req, &StatusData {
                startup_time: stats.startup_time,
                scan_time: stats.scan_time,
                load_avg_1min: stats.machine.load_avg_1min,
                load_avg_5min: stats.machine.load_avg_5min,
                load_avg_15min: stats.machine.load_avg_15min,
                boot_time: stats.machine.boot_time,
            })),
            "/all_processes.json" => Ok(http::reply_json(req, &ProcessData {
                boot_time: stats.machine.boot_time,
                all: &stats.processes.all,
            })),
            "/details.json" => Ok(http::reply_json(req, &DetailsData {
                startup_time: stats.startup_time,
                scan_time: stats.scan_time,
                machine: &stats.machine,
            })),
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
