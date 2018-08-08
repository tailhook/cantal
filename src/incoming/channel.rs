use std::collections::HashSet;
use std::time::Duration;
use std::sync::Arc;

use futures::{Stream, Async, Future};
use futures::sync::mpsc;
use futures::future::{loop_fn, Loop};
use tk_easyloop::{spawn, timeout};
use tk_http::websocket::Packet;
use serde_json::{to_string};

use frontend::graphql;
use incoming::{Subscription, IncomingImpl};
use incoming::dispatcher::{OutputMessage};


#[derive(Clone, Debug)]
pub struct Sender(pub(in incoming) mpsc::UnboundedSender<Subscription>);

#[derive(Debug)]
pub struct Receiver(mpsc::UnboundedReceiver<Subscription>);

pub fn new() -> (Sender, Receiver) {
    let (tx, rx) = mpsc::unbounded();
    return (Sender(tx), Receiver(rx));
}

fn insert(triggered: &mut HashSet<Subscription>, s: Subscription) {
    use incoming::Subscription::*;
    match s {
        Scan => { triggered.insert(Status); }
        Status => {},
        Peers => {},
    }
    triggered.insert(s);
}

impl Receiver {
    pub(in incoming) fn start(self,
        inc: &Arc<IncomingImpl>, ctx: &graphql::Context)
    {
        let inc = inc.clone();
        let ctx = ctx.clone();
        let Receiver(me) = self;
        let me = me.fuse();
        spawn(loop_fn((inc, ctx, me), move |(inc, ctx, me)| {
            me.into_future()
            .map_err(|((), _stream)| {
                error!("Subscription sender closed");
            })
            .and_then(|data| {
                timeout(Duration::from_millis(10))
                .map_err(|_| unreachable!())
                .map(move |_| data)
            })
            .and_then(move |(item, mut me)| {
                let first = match item {
                    None => return Ok(Loop::Break(())),
                    Some(item) => item,
                };
                let mut buf = HashSet::new();
                insert(&mut buf, first);
                loop {
                    match me.poll() {
                        Err(e) => return Err(e),
                        Ok(Async::Ready(Some(x))) => {
                            insert(&mut buf, x);
                        }
                        Ok(Async::Ready(None)) => break,
                        Ok(Async::NotReady) => break,
                    }
                }
                for item in buf {
                    dispatch(&item, &inc, &ctx);
                }
                Ok(Loop::Continue((inc, ctx, me)))
            })
            .and_then(|data| {
                timeout(Duration::from_millis(100))
                .map_err(|_| unreachable!())
                .map(move |_| data)
            })
        }))
    }
}

fn dispatch(subscription: &Subscription,
    inc: &IncomingImpl, ctx: &graphql::Context)
{
    let conns = inc.state.lock().expect("lock is not poisoned")
        .subscriptions.get(&subscription)
        .map(|x| x.clone())
        .unwrap_or_else(HashSet::new);
    for conn in conns {
        let clock = conn.0.state.lock().expect("lock is not poisoned");
        let items = clock.subscriptions.get(&subscription);
        for (id, input) in items.iter().flat_map(|x| *x) {
            let result = graphql::ws_response(ctx, &input);
            let packet = Packet::Text(
                to_string(&OutputMessage::Data {
                    id: id.clone(),
                    payload: result,
                })
                .expect("can serialize"));
            conn.0.tx.unbounded_send(packet)
                .map_err(|e| {
                    trace!("can't reply with ack: {}", e)
                }).ok();
        }
    }
}
