use std::sync::{Arc, Mutex, MutexGuard};
use std::sync::atomic::{Ordering, AtomicBool};

use futures::sync::mpsc::{unbounded, UnboundedSender, UnboundedReceiver};
use tk_easyloop::spawn;

use id::Id;
use gossip::Gossip;
use frontend::last_values::{RemoteMetric};

mod connection;
mod hostname;
mod manager;
mod tracking;

pub use self::hostname::Hostname;

#[derive(Debug)]
pub enum Message {
    Start,
    PeersUpdated,
}

#[derive(Debug)]
pub struct Init {
    rx: UnboundedReceiver<Message>,
    shared: Shared,
}

#[derive(Debug, Clone)]
pub struct Remote {
    tx: UnboundedSender<Message>,
    shared: Shared,
}

#[derive(Debug)]
pub struct SharedState {
    dead_connections: Vec<Id>,
    tracking: tracking::Tracking,
}

#[derive(Debug)]
pub struct SharedInfo {
    started: AtomicBool,
    state: Mutex<SharedState>
}

pub type Shared = Arc<SharedInfo>;

pub fn init() -> (Remote, Init) {
    let (tx, rx) = unbounded();
    let shared = Arc::new(SharedInfo {
        started: AtomicBool::new(false),
        state: Mutex::new(SharedState {
            dead_connections: Vec::new(),
            tracking: tracking::Tracking::new(),
        }),
    });
    return (Remote { tx, shared: shared.clone() }, Init { rx, shared });
}

impl Init {
    pub fn spawn(self, gossip: &Gossip) {
        spawn(manager::Manager::new(self.rx, gossip, &self.shared));
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
    pub fn started(&self) -> bool {
        self.shared.started.load(Ordering::SeqCst)
    }
    pub fn state(&self) -> MutexGuard<SharedState> {
        self.shared.state.lock().expect("remote shared state is fine")
    }
    pub fn query_remote<'x>(&self, filter: &tracking::Filter)
        -> Vec<RemoteMetric>
    {
        let ref mut trk = self.state().tracking;
        trk.add_timed_filter(filter);
        trk.get_values(filter)
    }
}
