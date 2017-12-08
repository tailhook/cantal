use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use std::net::SocketAddr;

use gossip::Gossip;
use frontend::{Request};
use frontend::routing::Format;
use frontend::quick_reply::{read_json, respond};

#[derive(Deserialize)]
struct Query {
    addr: SocketAddr,
}

#[derive(Serialize)]
struct Response {
    ok: bool,
}

pub fn add_host<S: 'static>(gossip: &Gossip, format: Format)
    -> Request<S>
{
    let gossip = gossip.clone();
    read_json(move |input: Query, e| {
        gossip.add_host(input.addr);
        // TODO(tailhook) wait for the host to be actually added
        Box::new(respond(e, format, &Response { ok: true }))
    })
}
