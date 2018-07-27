use std::sync::{Arc, Mutex};

use futures::{Future, Stream, Async};
use futures::stream::futures_unordered::FuturesUnordered;
use futures::sync::mpsc::UnboundedReceiver;

use remote::Message;
use remote::connection::Connection;
use remote::{Shared, SharedState};
use gossip::{Gossip, Peer};


pub struct Manager {
    rx: UnboundedReceiver<Message>,
    gossip: Gossip,
    state: Option<State>,
}

pub struct State {
    futures: FuturesUnordered<Connection>,
    active: HashSet<Id>,
    shared: Shared,
}

impl Manager {
    pub fn new(rx: UnboundedReceiver<Message>, gossip: &Gossip) -> Manager {
        Manager {
            rx,
            gossip: gossip.clone(),
            state: None,
        }
    }

    fn receive_messages(&mut self) {
        use remote::Message::*;
        loop {
            let msg = match self.rx.poll() {
                Ok(Async::Ready(Some(msg))) => msg,
                Ok(Async::NotReady) => return,
                Ok(Async::Ready(None)) | Err(()) => {
                    panic!("remote input channel is dropped");
                }
            };
            match msg {
                Start => {
                    if self.state.is_none() {
                        let mut state = State {
                            shared: Arc::new(Mutex::new(SharedState {
                                dead_connections: Vec::new(),
                            })),
                            active: HashSet::new(),
                            futures: FuturesUnordered::new(),
                        };
                        state.check_connections(self.gossip.get_peers());
                        self.state = Some(state);
                    }
                }
                PeersUpdated => {
                    if let Some(ref mut state) = self.state {
                        state.check_connections(self.gossip.get_peers());
                    } else {
                        // skip it
                    }
                }
            }
        }
    }
}

impl Future for Manager {
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Result<Async<()>, ()> {
        self.receive_messages();
        unimplemented!();
    }
}

impl State {
    fn check_connections(&mut self, peers: Vec<Arc<Peer>>) {
        for peer in &peers {
            if self.active.contains(peer.id) {
                continue;
            }
            if let Some(addr) = peer.primary_addr {
                self.futures.push(
                    Connection::new(&peer.id, addr, &self.shared));
                self.active.insert(peer.id.clone());
            } else {
                // TODO(tailhook) add to failed hosts
            }
        }
    }
}
