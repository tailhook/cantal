use std::net::SocketAddr;

use futures::sync::mpsc::UnboundedSender;

use gossip::command::Command;


/// A struct representing public interface for gossip
#[derive(Clone)]
pub struct Gossip {
    sender: UnboundedSender<Command>,
}

pub fn new(tx: UnboundedSender<Command>) -> Gossip {
    Gossip {
        sender: tx,
    }
}

impl Gossip {
    /// Asynchronous adds host to the list of known hosts
    pub fn add_host(&self, addr: SocketAddr) {
        self.sender.send(Command::AddHost(addr));
    }
}
