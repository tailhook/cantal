use std::sync::RwLock;
use std::time::Duration;
use serialize::json::Json;
use serialize::json::as_pretty_json;

use super::aio;
use super::scan;
use super::staticfiles;
use super::aio::http;
use super::stats::Stats;


#[derive(Encodable)]
struct StatusData<'a> {
    startup_time: u64,
    scan_time: u64,
    machine: &'a scan::machine::MachineStats,
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
    } else if req.uri() == "/status.json" {
        let stats = stats.read().unwrap();
        let mut builder = http::ResponseBuilder::new(req, http::Status::Ok);
        builder.set_body(format!("{}", as_pretty_json(&StatusData {
            startup_time: stats.startup_time,
            scan_time: stats.scan_time,
            machine: &stats.machine,
        })).into_bytes());
        Ok(builder.take())
    } else if req.uri() == "/all_processes.json" {
        let stats = stats.read().unwrap();
        let mut builder = http::ResponseBuilder::new(req, http::Status::Ok);
        builder.set_body(format!("{}", as_pretty_json(&ProcessData {
            boot_time: stats.machine.boot_time,
            all: &stats.processes.all,
            })).into_bytes());
        Ok(builder.take())
    } else {
        return Err(http::Error::NotFound);
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
