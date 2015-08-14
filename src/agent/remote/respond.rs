use std::str::from_utf8;
use std::collections::{HashMap};

use time::{SteadyTime};
use probor;
use rustc_serialize::json;

use query::{Rule, query_history};
use super::super::stats::Stats;
use super::super::server::Context;
use super::super::http::{Request, BadRequest};
use super::super::http;
use super::aggregate;
use super::{Peers, ensure_started};
use super::{DATA_POINTS};
use super::super::server::{MAX_OUTPUT_BUFFER};
use super::super::websock::InputMessage as OutputMessage;
use super::super::websock::{write_binary};
use super::super::deps::LockedDeps;
use super::super::ioutil::Poll;


pub fn serve_query_by_host(req: &Request, context: &mut Context)
    -> Result<http::Response, Box<http::Error>>
{
    #[derive(RustcDecodable)]
    struct Query {
        rules: HashMap<String, Rule>,
    }
    from_utf8(&req.body)
    .map_err(|_| BadRequest::err("Bad utf-8 encoding"))
    .and_then(|s| json::decode::<Query>(s)
    .map_err(|_| BadRequest::err("Failed to decode query")))
    .and_then(|query| {
        ensure_started(context);

        let mut resp = {
            let mut peerguard = context.deps.write::<Option<Peers>>();
            let mut peers = peerguard.as_mut().unwrap();

            for (_, rule) in query.rules.iter() {
                let ts = SteadyTime::now();
                if let Some(ts_ref) = peers.subscriptions.get_mut(&rule.series) {
                    *ts_ref = ts;
                    continue;
                }
                // TODO(tailhook) may optimize this rule.clone()
                let subscr = OutputMessage::Subscribe(
                    rule.series.clone(), DATA_POINTS);
                let msg = probor::to_buf(&subscr);
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
                            write_binary(&mut wsock.output, &msg);
                            if start {
                                eloop.modify(&wsock.sock, *tok, true, true);
                            }
                        }
                        Some(peer)
                    }).unwrap()
                }
                peers.subscriptions.insert(rule.series.clone(), ts);
            }
            aggregate::query(&query.rules, peers)
        };

        {
            let stats = context.deps.read::<Stats>();
            let mut dict = HashMap::new();
            for (name, ref rule) in query.rules.iter() {
                dict.insert(name.clone(), query_history(rule, &stats.history));
            }
            // TODO(tailhook) find myself ip addr
            resp.insert("myself".to_string(), dict);
        }

        Ok(http::Response::probor(&resp))
    })
}
