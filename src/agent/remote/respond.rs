use std::str::from_utf8;

use time::{SteadyTime};
use rustc_serialize::json;

use super::super::server::Context;
use super::super::http::{Request, BadRequest};
use super::super::http;
use super::super::rules;
use super::aggregate;
use super::super::rules::{Query, RawQuery, RawRule, RawResult};
use super::{Peers, ensure_started};
use super::{DATA_POINTS};
use super::super::server::{MAX_OUTPUT_BUFFER};
use super::super::websock::InputMessage as OutputMessage;
use super::super::websock::{write_text};
use super::super::deps::LockedDeps;
use super::super::ioutil::Poll;


#[derive(RustcDecodable, RustcEncodable)]
pub struct HostStats {
    addr: String,
    values: RawResult,
}


pub fn serve_query_raw(req: &Request, context: &mut Context)
    -> Result<http::Response, Box<http::Error>>
{
    from_utf8(&req.body)
    .map_err(|_| BadRequest::err("Bad utf-8 encoding"))
    .and_then(|s| json::decode::<RawQuery>(s)
    .map_err(|_| BadRequest::err("Failed to decode query")))
    .and_then(|query| {
        ensure_started(context);

        let mut peerguard = context.deps.write::<Peers>();
        let mut peers = &mut *peerguard;

        let response: Vec<_> = peers.peers.iter().map(|peer| HostStats {
            addr: peer.addr.to_string(),
            values: rules::query_raw(query.rules.iter(),
                              DATA_POINTS, &peer.history),
        }).collect();

        for rule in query.rules.into_iter() {
            let ts = SteadyTime::now();
            if let Some(ts_ref) = peers.subscriptions.get_mut(&rule) {
                *ts_ref = ts;
                continue;
            }
            // TODO(tailhook) may optimize this rule.clone()
            let subscr = OutputMessage::Subscribe(rule.clone(), DATA_POINTS);
            let msg = json::encode(&subscr).unwrap();
            let ref mut addresses = &mut peers.addresses;
            let ref mut peerlist = &mut peers.peers;
            let ref mut eloop = context.eloop;
            for tok in addresses.values() {
                peerlist.replace_with(*tok, |mut peer| {
                    if let Some(ref mut wsock) = peer.connection {
                        if wsock.output.len() > MAX_OUTPUT_BUFFER {
                            debug!("Websocket buffer overflow");
                            eloop.remove(&wsock.sock);
                            return None;
                        }
                        let start = wsock.output.len() == 0;
                        write_text(&mut wsock.output, &msg);
                        if start {
                            eloop.modify(&wsock.sock, *tok, true, true);
                        }
                    }
                    Some(peer)
                }).unwrap()
            }
            peers.subscriptions.insert(rule, ts);
        }

        Ok(http::Response::json(&response))
    })
}

pub fn serve_query_by_host(req: &Request, context: &mut Context)
    -> Result<http::Response, Box<http::Error>>
{
    from_utf8(&req.body)
    .map_err(|_| BadRequest::err("Bad utf-8 encoding"))
    .and_then(|s| json::decode::<Query>(s)
    .map_err(|_| BadRequest::err("Failed to decode query")))
    .and_then(|query| {
        ensure_started(context);

        let mut peerguard = context.deps.write::<Peers>();
        let mut peers = &mut *peerguard;

        for (_, qrule) in query.rules.iter() {
            let rule = RawRule {
                source: qrule.source,
                condition: qrule.condition.clone(),
                key: qrule.key.clone(),
            };
            let ts = SteadyTime::now();
            if let Some(ts_ref) = peers.subscriptions.get_mut(&rule) {
                *ts_ref = ts;
                continue;
            }
            // TODO(tailhook) may optimize this rule.clone()
            let subscr = OutputMessage::Subscribe(rule.clone(), DATA_POINTS);
            let msg = json::encode(&subscr).unwrap();
            let ref mut addresses = &mut peers.addresses;
            let ref mut peerlist = &mut peers.peers;
            let ref mut eloop = context.eloop;
            for tok in addresses.values() {
                peerlist.replace_with(*tok, |mut peer| {
                    if let Some(ref mut wsock) = peer.connection {
                        if wsock.output.len() > MAX_OUTPUT_BUFFER {
                            debug!("Websocket buffer overflow");
                            eloop.remove(&wsock.sock);
                            return None;
                        }
                        let start = wsock.output.len() == 0;
                        write_text(&mut wsock.output, &msg);
                        if start {
                            eloop.modify(&wsock.sock, *tok, true, true);
                        }
                    }
                    Some(peer)
                }).unwrap()
            }
            peers.subscriptions.insert(rule, ts);
        }

        Ok(http::Response::json(&aggregate::query(&query, peers)))
    })
}
