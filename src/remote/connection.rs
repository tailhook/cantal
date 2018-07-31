use std::sync::Arc;
use std::mem;
use std::net::SocketAddr;

use remote::Shared;
use id::Id;

use futures::{Future, Async, Stream};
use futures::future::{FutureResult, ok};
use tokio::net::{TcpStream, ConnectFuture};
use tk_http::websocket::client::{HandshakeProto, SimpleAuthorizer};
use tk_http::websocket::{Loop, Frame, Error, Dispatcher as Disp, Config};
use tk_http::websocket::{Packet};
use tk_easyloop::handle;

lazy_static! {
    static ref CONFIG: Arc<Config> = Config::new()
        .done();
}


pub enum State {
    Connecting(ConnectFuture),
    Handshake(HandshakeProto<TcpStream, SimpleAuthorizer>),
    Active(Loop<TcpStream, Packetize, Dispatcher>),
    Void,
}

pub struct Connection {
    id: Id,
    shared: Shared,
    state: State,
}

pub struct Dispatcher {
}

pub struct Packetize;

impl Disp for Dispatcher {
    type Future = FutureResult<(), Error>;
    fn frame(&mut self, frame: &Frame) -> FutureResult<(), Error> {
        debug!("Frame arrived: {:?}", frame);
        ok(())
    }
}

impl Stream for Packetize {
    type Item = Packet;
    type Error = &'static str;
    fn poll(&mut self) -> Result<Async<Option<Packet>>, &'static str> {
        Ok(Async::NotReady)
    }
}


impl Drop for Connection {
    fn drop(&mut self) {
        let mut state = self.shared.state.lock()
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
                        self.state = Handshake(
                            HandshakeProto::new(conn, SimpleAuthorizer::new(
                                "cantal.internal", "/graphql")));
                    }
                    Err(e) => {
                        error!("Error connecting to {}: {}", self.id, e);
                        return Err(());
                    }
                }
                Handshake(mut f) => match f.poll() {
                    Ok(Async::NotReady) => {
                        self.state = Handshake(f);
                        return Ok(Async::NotReady);
                    }
                    Ok(Async::Ready((out, inp, ()))) => {
                        self.state = Active(Loop::client(out, inp, Packetize,
                            Dispatcher {}, &*CONFIG, &handle()));
                    }
                    Err(e) => {
                        error!("Error handshaking with {}: {}", self.id, e);
                        return Err(());
                    }
                }
                Active(mut f) => match f.poll() {
                    Ok(Async::NotReady) => {
                        self.state = Active(f);
                        return Ok(Async::NotReady);
                    }
                    Ok(Async::Ready(())) => {
                        self.state = Void;
                        return Ok(Async::Ready(()));
                    }
                    Err(e) => {
                        warn!("Connection to {} dropped: {}", self.id, e);
                        return Err(());
                    }
                }
                Void => unreachable!("void connection state"),
            };
        }
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
