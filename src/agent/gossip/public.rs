use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use futures::sync::mpsc::UnboundedSender;

use gossip::command::Command;
use gossip::info::Info;


/// A struct representing public interface for gossip
///
/// This structure may represent either working gossip subsystem or a no-op
/// methods.
#[derive(Clone)]
pub struct Gossip {
    sender: Option<UnboundedSender<Command>>,
    info: Arc<Mutex<Info>>,
}

pub fn new(tx: UnboundedSender<Command>, info: &Arc<Mutex<Info>>) -> Gossip {
    Gossip {
        sender: Some(tx),
        info: info.clone(),
    }
}

pub fn noop() -> Gossip {
    Gossip {
        sender: None,
        info: Arc::new(Mutex::new(Info::new())),
    }
}

impl Gossip {
    /// Asynchronous adds host to the list of known hosts
    pub fn add_host(&self, addr: SocketAddr) {
        if let Some(ref sender) = self.sender {
            sender.send(Command::AddHost(addr));
        }
    }
    /// Number of peers total and those having "remote" enabled
    pub fn get_peer_numbers(&self) -> (usize, usize) {
        let info = self.info.lock().expect("gossip is not poisoned");
        let num_remote = info.peers.iter()
            .filter(|&(_, peer)| {
                peer.report.as_ref()
                .map(|&(_, ref x)| x.has_remote)
                .unwrap_or(false)
            })
            .count();
        return (info.peers.len(), num_remote);
    }
}
