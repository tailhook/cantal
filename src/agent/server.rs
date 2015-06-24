use std::str::from_utf8;
use std::sync::RwLock;
use std::collections::HashMap;
use rustc_serialize::json;
use rustc_serialize::json::ToJson;

use mio;

use super::aio;
use super::scan;
use super::staticfiles;
use super::aio::http;
use super::stats::{Stats};
use super::storage::{StorageStats};
use super::rules::{Query, query};
use super::p2p::Command;


#[derive(RustcEncodable)]
struct StatusData {
    pub startup_time: u64,
    pub scan_duration: u32,
    pub storage: StorageStats,
    pub boot_time: Option<u64>,
}

#[derive(RustcEncodable)]
struct ProcessesData<'a> {
    boot_time: Option<u64>,
    all: &'a Vec<scan::processes::MinimalProcess>,
}

fn handle_request(stats: &RwLock<Stats>, req: &http::Request,
    gossip_cmd: mio::Sender<Command>)
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
                storage: stats.storage,
                boot_time: stats.boot_time,
            })),
            "/all_processes.json" => Ok(http::reply_json(req, &ProcessesData {
                boot_time: stats.boot_time,
                all: &stats.processes,
            })),
            "/all_metrics.json" => Ok(http::reply_json(req,
                &stats.history.tip.keys()
                .chain(stats.history.fine.keys())
                .chain(stats.history.coarse.keys())
                .collect::<Vec<_>>()
                .to_json()
            )),
            "/all_peers.json" => Ok(http::reply_json(req,
                &json::Json::Object(vec![
                    (String::from("peers"), json::Json::Array(
                        stats.gossip.read().unwrap().peers.values()
                        .map(ToJson::to_json)
                        .collect())),
                ].into_iter().collect()
            ))),
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
            "/add_host.json" => {
                #[derive(RustcDecodable)]
                struct Query {
                    ip: String,
                }
                from_utf8(req.body.unwrap_or(b""))
               .map_err(|_| http::Error::BadRequest("Bad utf-8 encoding"))
               .and_then(|x| json::decode(x)
               .map_err(|e| error!("Error parsing query: {:?}", e))
               .map_err(|_| http::Error::ServerError("Request format error")))
               .and_then(|x: Query| x.ip.parse()
               .map_err(|_| http::Error::BadRequest("Can't parse IP address")))
               .and_then(|x| gossip_cmd.send(Command::AddGossipHost(x))
               .map_err(|e| error!("Error sending to p2p loop: {:?}", e))
               .map_err(|_| http::Error::ServerError("Notify Error")))
               .and_then(|_| {
                    Ok(http::reply_json(req, &vec![
                        (String::from("ok"), true)
                    ].into_iter().collect::<HashMap<_, _>>().to_json()))
                })
            }
            _ => Err(http::Error::NotFound),
        }
    }
}


pub fn run_server(stats: &RwLock<Stats>, host: &str, port: u16,
    gossip_cmd: mio::Sender<Command>)
    -> Result<(), String>
{
    let handler: &for<'b> Fn(&'b aio::http::Request<'b>)
        -> Result<aio::http::Response, aio::http::Error>
        = &|req| {
        handle_request(stats, req, gossip_cmd.clone())
    };
    let mut main = try!(aio::MainLoop::new()
        .map_err(|e| format!("Can't create main loop: {}", e)));
    try!(main.add_http_server(host, port, handler)
        .map_err(|e| format!("Can't bind {}:{}: {}", host, port, e)));
    main.run();
}
