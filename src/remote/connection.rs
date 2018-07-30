use std::mem;
use std::net::SocketAddr;

use remote::Shared;
use id::Id;

use futures::{Future, Async};
use tokio::net::{TcpStream, ConnectFuture};

pub enum State {
    Connecting(ConnectFuture),
    Connected(TcpStream),
    Void,
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
        use self::State::*;
        loop {
            match mem::replace(&mut self.state, Void) {
                Connecting(mut f) => match f.poll() {
                    Ok(Async::NotReady) => {
                        self.state = Connecting(f);
                        return Ok(Async::NotReady);
                    }
                    Ok(Async::Ready(conn)) => {
                        self.state = Connected(conn);
                    }
                    Err(e) => {
                        error!("Error connecting to {}: {}", self.id, e);
                        return Err(());
                    }
                }
                Connected(conn) => unimplemented!("connected"),
                Void => unreachable!("void connection state"),
            };
        }
        Ok(Async::NotReady)
    }
}

impl Connection {
    pub fn new(id: &Id, addr: SocketAddr, shared: &Shared) -> Connection {
        info!("Connecting to {} via {}", id, addr);
        Connection {
            id: id.clone(),
            shared: shared.clone(),
            state: State::Connecting(TcpStream::connect(&addr)),
        }
    }
}
