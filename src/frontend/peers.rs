use std::sync::{Arc};

use frontend::{Request, graphql::ContextRef};
use frontend::routing::Format;
use frontend::quick_reply::{reply, respond};
use gossip::{Peer, Gossip};


#[derive(Serialize)]
struct Peers {
    peers: Vec<Arc<Peer>>,
}

#[derive(GraphQLInputObject)]
#[graphql(name="PeerFilter", description="Filter for peers")]
pub struct Filter {
    pub has_remote: Option<bool>,
}

pub fn serve<S: 'static>(gossip: &Gossip, format: Format)
    -> Request<S>
{
    let gossip = gossip.clone();
    reply(move |e| {
        Box::new(respond(e, format,
            &Peers {
                peers: gossip.get_peers(),
            }
        ))
    })
}

pub fn serve_only_remote<S: 'static>(gossip: &Gossip, format: Format)
    -> Request<S>
{
    let gossip = gossip.clone();
    reply(move |e| {
        Box::new(respond(e, format,
            &Peers {
                peers: gossip.get_peers().into_iter().filter(|x| {
                    x.report.as_ref()
                        .map(|&(_, ref r)| r.has_remote)
                        .unwrap_or(false)
                }).collect(),
            }
        ))
    })
}

pub fn get(context: &ContextRef, filter: Option<Filter>) -> Vec<Arc<Peer>> {
    let mut peers = context.gossip.get_peers();
    if let Some(filter) = filter {
        peers.retain(|p| {
            if let Some(remote) = filter.has_remote {
                if remote != p.has_remote() {
                    return false;
                }
            }
            return true;
        })
    }
    return peers;
}
