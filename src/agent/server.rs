use std::str::from_utf8;
use std::sync::RwLock;
use std::collections::HashMap;
use rustc_serialize::json;
use rustc_serialize::json::ToJson;

use super::aio;
use super::scan;
use super::staticfiles;
use super::aio::http;
use super::stats::{Stats};
use super::rules::{Query, query};


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
struct ProcessesData<'a> {
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
                   Ok(http::reply_json(req, &vec![
                    (String::from("dataset"), try!(query(&r, &*stats))),
                    (String::from("tip_timestamp"), h.tip_timestamp.to_json()),
                    (String::from("fine_timestamps"), h.fine_timestamps
                        .iter().cloned().collect::<Vec<_>>().to_json()),
                    (String::from("coarse_timestamps"), h.coarse_timestamps
                        .iter().cloned().collect::<Vec<_>>().to_json()),
                   ].into_iter().collect::<HashMap<_,_>>().to_json()))
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
