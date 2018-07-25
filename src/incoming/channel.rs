use std::collections::HashSet;
use std::time::Duration;

use futures::{Stream, Async, Future};
use futures::sync::mpsc;
use futures::future::{loop_fn, Loop};

use tk_easyloop::{spawn, timeout};
use incoming::{Subscription, Incoming};


#[derive(Clone, Debug)]
pub struct Sender(mpsc::UnboundedSender<Subscription>);

#[derive(Debug)]
pub struct Receiver(mpsc::UnboundedReceiver<Subscription>);

pub fn new() -> (Sender, Receiver) {
    let (tx, rx) = mpsc::unbounded();
    return (Sender(tx), Receiver(rx));
}

impl Sender {
    pub fn trigger(&self, subscription: Subscription) {
        self.0.unbounded_send(subscription)
            .map_err(|e| error!("Can't trigger subscription: {}", e))
            .ok();
    }
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
    pub fn start(self, inc: &Incoming) {
        let inc = inc.clone();
        let Receiver(me) = self;
        let me = me.fuse();
        spawn(loop_fn((inc, me), move |(inc, me)| {
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
                    inc.trigger(&item);
                }
                Ok(Loop::Continue((inc, me)))
            })
            .and_then(|data| {
                timeout(Duration::from_millis(100))
                .map_err(|_| unreachable!())
                .map(move |_| data)
            })
        }))
    }
}
