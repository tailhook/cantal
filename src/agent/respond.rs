use std::str::from_utf8;
use std::collections::HashMap;

use rustc_serialize::json;
use rustc_serialize::json::ToJson;
use rustc_serialize::hex::ToHex;
use self_meter;
use probor;

use query::{Rule, query_history, Dataset};
use history::TimeStamp;
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
struct ProcessesData<'a> {
    boot_time: Option<u64>,
    all: &'a Vec<scan::processes::MinimalProcess>,
}

pub fn serve_status(_req: &Request, context: &mut Context)
    -> Result<http::Response, Box<http::Error>>
{
    #[derive(RustcEncodable)]
    struct StatusData {
        startup_time: u64,
        scan_duration: u32,
        storage: StorageStats,
        boot_time: Option<u64>,
        self_report: Option<self_meter::Report>,
        threads_report: HashMap<String, self_meter::ThreadReport>,
    }
    let (me, thr) = {
        let meter = context.deps.lock::<self_meter::Meter>();
        (meter.report(),
         meter.thread_report()
            .map(|x| x.map(|(k, v)| (k.to_string(), v)).collect())
            .unwrap_or(HashMap::new()))
    };
    let stats: &Stats = &*context.deps.read();
    Ok(http::Response::json(&StatusData {
            startup_time: stats.startup_time,
            scan_duration: stats.scan_duration,
            storage: stats.storage,
            boot_time: stats.boot_time,
            self_report: me,
            threads_report: thr,
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

pub fn serve_sockets(_req: &Request, context: &mut Context)
    -> Result<http::Response, Box<http::Error>>
{
    let stats: &Stats = &*context.deps.read();
    Ok(http::Response::json(&stats.connections))
}

pub fn serve_metrics(_req: &Request, context: &mut Context)
    -> Result<http::Response, Box<http::Error>>
{
    struct Response<'x> {
        metrics: Vec<(&'x Key, TimeStamp, Value)>,
    }
    impl<'x> probor::Encodable for Response<'x> {
        fn encode<W: probor::Output>(&self, e: &mut probor::Encoder<W>)
            -> Result<(), probor::EncodeError>
        {
            probor_enc_struct!(e, self, {
                metrics => (),
            });
            Ok(())
        }
    }
    use history::{Key, TimeStamp};
    use cantal::Value;
    let stats: &Stats = &*context.deps.read();
    let ref fts = stats.history.fine.timestamps;
    let fage = stats.history.fine.age;
    let vec: Vec<(&Key, TimeStamp, Value)> =
        stats.history.tip.values.iter().map(|(k, &(ts, ref v))| (k, ts, v.clone()))
        .chain(stats.history.fine.values.iter().map(
            |(k, v)| (k, fts[(fage - v.age()) as usize].0, v.tip_value())))
        .collect();
    Ok(http::Response::probor(&Response {
        metrics: vec,
    }))
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
        id: String,
        current_addr: Option<String>,
        connected: bool,
        last_beacon_time: Option<u64>,
        last_beacon: Option<Beacon>,
        last_attempt: Option<(TimeStamp, &'static str)>,
    }
    let response = if let Some(ref peers) = *context.deps.lock::<Option<Peers>>() {
        let mut result = Vec::new();
        for p in peers.peers.iter() {
            result.push(PeerInfo {
                id: p.id.to_hex(),
                current_addr: p.current_addr.map(|x| x.to_string()),
                connected: p.connected(),
                last_beacon_time: p.last_beacon.as_ref().map(|x| x.0),
                last_beacon: p.last_beacon.as_ref().map(|x| x.1.clone()),
                last_attempt: p.last_attempt,
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

    struct Response {
        values: HashMap<String, Dataset>,
    }

    impl<'x> probor::Encodable for Response {
        fn encode<W: probor::Output>(&self, e: &mut probor::Encoder<W>)
            -> Result<(), probor::EncodeError>
        {
            probor_enc_struct!(e, self, {
                values => (),
            });
            Ok(())
        }
    }


    let stats: &Stats = &*context.deps.read();
    let h = &stats.history;
    from_utf8(&req.body)
       .map_err(|_| BadRequest::err("Bad utf-8 encoding"))
       .and_then(|s| json::decode::<Query>(s)
       .map_err(|e| debug!("Decoding error {}", e))
       .map_err(|_| BadRequest::err("Failed to decode query")))
       .and_then(|r| {
           Ok(http::Response::probor(&Response {
                values: r.rules.into_iter()
                    .map(|(key, rule)| (key, query_history(&rule, h)))
                    .collect(),
           }))
       })
}
