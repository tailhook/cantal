use std::net::SocketAddr;

use remote::Shared;
use id::Id;

use futures::{Future, Async};

pub enum State {
}

pub struct Connection {
    id: Id,
    shared: Shared,
    state: State,
}

impl Drop for Connection {
    fn drop(&mut self) {
        let mut state = self.shared.lock()
            .expect("shared object is not poisoned");
        state.dead_connections.push(self.id.clone());
    }
}

impl Future for Connection {
    type Item = ();
    type Error = ();
    fn poll(&mut self) -> Result<Async<()>, ()> {
        unimplemented!();
    }
}

impl Connection {
    pub fn new(id: &Id, addr: SocketAddr, shared: &Shared) -> Connection {
        Connection {
            id: id.clone(),
            shared: shared.clone(),
            state: unimplemented!(),
        }
    }
}
