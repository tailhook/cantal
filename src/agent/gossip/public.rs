use std::net::SocketAddr;

use futures::sync::mpsc::UnboundedSender;

use gossip::command::Command;


/// A struct representing public interface for gossip
///
/// This structure may represent either working gossip subsystem or a no-op
/// methods.
#[derive(Clone)]
pub struct Gossip {
    sender: Option<UnboundedSender<Command>>,
}

pub fn new(tx: UnboundedSender<Command>) -> Gossip {
    Gossip {
        sender: Some(tx),
    }
}

pub fn noop() -> Gossip {
    Gossip {
        sender: None
    }
}

impl Gossip {
    /// Asynchronous adds host to the list of known hosts
    pub fn add_host(&self, addr: SocketAddr) {
        if let Some(ref sender) = self.sender {
            sender.send(Command::AddHost(addr));
        }
    }
}
