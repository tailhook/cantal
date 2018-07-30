use std::sync::{Arc, Mutex};

use futures::sync::mpsc::{unbounded, UnboundedSender, UnboundedReceiver};
use tk_easyloop::spawn;

use id::Id;
use gossip::Gossip;

mod manager;
mod connection;

#[derive(Debug)]
pub enum Message {
    Start,
    PeersUpdated,
}

#[derive(Debug)]
pub struct Init {
    rx: UnboundedReceiver<Message>,
}

#[derive(Debug, Clone)]
pub struct Remote {
    tx: UnboundedSender<Message>,
}

pub struct SharedState {
    dead_connections: Vec<Id>,
}

pub type Shared = Arc<Mutex<SharedState>>;

pub fn init() -> (Remote, Init) {
    let (tx, rx) = unbounded();
    return (Remote { tx }, Init { rx });
}

impl Init {
    pub fn spawn(self, gossip: &Gossip) {
        spawn(manager::Manager::new(self.rx, gossip));
    }
}

impl Remote {
    pub fn peers_updated(&self) {
        self.tx.unbounded_send(Message::PeersUpdated)
            .map_err(|_| error!("can't send message to remote subsystem"))
            .ok();
    }
    pub fn start(&self) {
        self.tx.unbounded_send(Message::Start)
            .map_err(|_| error!("can't send message to remote subsystem"))
            .ok();
    }
}
