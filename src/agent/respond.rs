use std::str::from_utf8;
use std::collections::HashMap;

use rustc_serialize::json;
use rustc_serialize::json::ToJson;

use query::{Rule, query_history};
use super::http;
use super::scan;
use super::storage::{StorageStats};
use super::http::{Request, BadRequest};
use super::server::Context;
use super::stats::Stats;
use super::p2p::GossipStats;
use super::remote::{Peers};
use super::deps::LockedDeps;
use super::websock::Beacon;


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

pub fn serve_status(_req: &Request, context: &mut Context)
    -> Result<http::Response, Box<http::Error>>
{
    let stats: &Stats = &*context.deps.read();
    Ok(http::Response::json(&StatusData {
            startup_time: stats.startup_time,
            scan_duration: stats.scan_duration,
            storage: stats.storage,
            boot_time: stats.boot_time,
        }))
}

pub fn serve_processes(_req: &Request, context: &mut Context)
    -> Result<http::Response, Box<http::Error>>
{
    let stats: &Stats = &*context.deps.read();
    Ok(http::Response::json(&ProcessesData {
            boot_time: stats.boot_time,
            all: &stats.processes,
        }))
}

pub fn serve_metrics(_req: &Request, context: &mut Context)
    -> Result<http::Response, Box<http::Error>>
{
    let stats: &Stats = &*context.deps.read();
    Ok(http::Response::json(
            &stats.history.tip.iter()
            .chain(stats.history.fine.iter())
            .chain(stats.history.coarse.iter())
            .map(|(k, v)| (k, v.tip()))
            .collect::<Vec<_>>()
            .to_json()
        ))
}

pub fn serve_peers(_req: &Request, context: &mut Context)
    -> Result<http::Response, Box<http::Error>>
{
    let gossip: &GossipStats = &*context.deps.read();
    let resp = http::Response::json(
        &json::Json::Object(vec![
            (String::from("peers"), json::Json::Array(
                gossip.peers.values()
                .map(ToJson::to_json)
                .collect())),
        ].into_iter().collect()
       ));
    Ok(resp)
}

pub fn serve_peers_with_remote(_req: &Request, context: &mut Context)
    -> Result<http::Response, Box<http::Error>>
{
    let gossip: &GossipStats = &*context.deps.read();
    let resp = http::Response::json(
        &json::Json::Object(vec![
            (String::from("peers"), json::Json::Array(
                gossip.peers.values()
                .filter(|x| x.report.as_ref()
                             .map(|&(_, ref r)| r.has_remote)
                             .unwrap_or(false))
                .map(ToJson::to_json)
                .collect())),
        ].into_iter().collect()
       ));
    Ok(resp)
}

pub fn serve_remote_stats(_req: &Request, context: &mut Context)
    -> Result<http::Response, Box<http::Error>>
{
    #[derive(RustcEncodable)]
    struct Response {
        enabled: bool,
        peers: Vec<PeerInfo>,
    }
    #[derive(RustcEncodable)]
    struct PeerInfo {
        addr: String,
        connected: bool,
        last_beacon_time: Option<u64>,
        last_beacon: Option<Beacon>,
    }
    let response = if let Some(ref peers) = *context.deps.read::<Option<Peers>>() {
        let mut result = Vec::new();
        for p in peers.peers.iter() {
            result.push(PeerInfo {
                addr: p.addr.to_string(),
                connected: p.connected(),
                last_beacon_time: p.last_beacon.as_ref().map(|x| x.0),
                last_beacon: p.last_beacon.as_ref().map(|x| x.1.clone()),
            })
        }
        Response {
            enabled: true,
            peers: result,
        }
    } else {
        Response {
            enabled: false,
            peers: Vec::new(),
        }
    };
    let resp = http::Response::json(&response);
    Ok(resp)
}

pub fn serve_query(req: &Request, context: &mut Context)
    -> Result<http::Response, Box<http::Error>>
{
    #[derive(RustcDecodable)]
    struct Query {
        rules: HashMap<String, Rule>,
    }
    let stats: &Stats = &*context.deps.read();
    let h = &stats.history;
    from_utf8(&req.body)
       .map_err(|_| BadRequest::err("Bad utf-8 encoding"))
       .and_then(|s| json::decode::<Query>(s)
       .map_err(|_| BadRequest::err("Failed to decode query")))
       .and_then(|r| {
           Ok(http::Response::json(r.rules.into_iter().map(|(key, rule)| {
                (key, query_history(rule, h))
           }).collect()))
        })
}
