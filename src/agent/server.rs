use std::str::FromStr;
use std::str::from_utf8;
use std::sync::RwLock;
use rustc_serialize::json;
use rustc_serialize::json::Json;

use super::aio;
use super::scan;
use super::staticfiles;
use super::aio::http;
use super::util::tree_collect;
use super::stats::{Stats, Key};
use super::rules::{Query, query};
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
            })),
            "/all_processes.json" => Ok(http::reply_json(req, &ProcessesData {
                boot_time: stats.boot_time,
                all: &stats.processes,
            })),
            "/query.json"
            => from_utf8(req.body.unwrap_or(b""))
               .map_err(|_| http::Error::BadRequest("Bad utf-8 encoding"))
               .and_then(|s| json::decode::<Query>(s)
               .map_err(|_| http::Error::BadRequest("Failed to decode query")))
               .and_then(|r| {
                   Ok(http::reply_json(req, &try!(query(&r, &*stats))))
                }),
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
