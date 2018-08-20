use std::sync::Arc;
use std::mem;
use std::net::SocketAddr;

use remote::{Shared, Hostname};
use id::Id;
use serde_json;

use futures::{Future, Async, Stream};
use futures::sync::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use futures::future::{FutureResult, ok, err};
use frontend::graphql;
use tokio::net::{TcpStream, ConnectFuture};
use tk_http::websocket::client::{HandshakeProto, SimpleAuthorizer};
use tk_http::websocket::{Loop, Frame, Error, Dispatcher as Disp, Config};
use tk_http::websocket::{self, Packet};
use tk_easyloop::handle;

lazy_static! {
    static ref CONFIG: Arc<Config> = Config::new()
        .done();
}

static SUBSCRIBE_METRICS: &str = "
    subscription {
        metrics {
            key
            .. on IntegerMetric {
                value
            }
            .. on CounterMetric {
                value
            }
            .. on FloatMetric {
                value
            }
            .. on StateMetric {
                timestamp
                value
            }
        }
    }
";


pub enum State {
    Connecting(ConnectFuture, UnboundedReceiver<Packet>, Dispatcher),
    Handshake(HandshakeProto<TcpStream, SimpleAuthorizer>,
        UnboundedReceiver<Packet>, Dispatcher),
    Active(Loop<TcpStream, Packetize, Dispatcher>),
    Void,
}

pub struct Connection {
    id: Id,
    shared: Shared,
    state: State,
}

pub struct Dispatcher {
    tx: UnboundedSender<Packet>,
    hostname: Hostname,
    shared: Shared,
}

pub struct Packetize {
    rx: UnboundedReceiver<Packet>,
}

#[derive(Debug, Serialize)]
#[serde(tag="type", rename_all="snake_case")]
pub enum OutputMessage {
    ConnectionInit { payload: ConnectionParams },
    Start { payload: graphql::Input, id: String },
    Stop { id: String },
}

#[derive(Debug, Deserialize)]
#[serde(tag="type", rename_all="snake_case")]
pub enum InputMessage {
    ConnectionAck,
    Data { id: String, payload: GraphqlResult },
}

#[derive(Debug, Deserialize)]
pub struct GraphqlError {
    message: String,
    // TODO(tailhook) other fields
}

#[derive(Debug, Deserialize)]
pub struct GraphqlResult {
    #[serde(default, skip_serializing_if="Option::is_none")]
    pub data: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if="ErrorWrapper::is_empty")]
    pub errors: Vec<GraphqlError>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConnectionParams {
}

impl Disp for Dispatcher {
    type Future = FutureResult<(), Error>;
    fn frame(&mut self, frame: &Frame) -> FutureResult<(), Error> {
        match *frame {
            Frame::Binary(_) => {
                error!("Received binary frame");
            }
            Frame::Text(txt) => {
                let value = match serde_json::from_str(txt) {
                    Ok(val) => val,
                    Err(e) => {
                        error!("invalid data {:?}: {}", txt, e);
                        return err(websocket::Error::custom("invalid frame"));
                    }
                };
                match value {
                    InputMessage::ConnectionAck => {
                        self.tx.unbounded_send(Packet::Text(
                            serde_json::to_string(
                                &OutputMessage::Start {
                                    id: 1.to_string(),
                                    payload: graphql::Input {
                                        operation_name: None,
                                        variables: None,
                                        query: SUBSCRIBE_METRICS.to_string(),
                                    },
                                }
                            ).expect("serialize connection_init"),
                        )).ok();
                    }
                    InputMessage::Data { id, payload } => {
                        // TODO
                        unimplemented!();
                    }
                }
            }
            Frame::Close(code, reason) => {
                debug!("Closed, code {}: {:?}", code, reason);
                // TODO(tailhook) should we do anything?
            }
            _ => {
                error!("Bad frame received: {:?}", frame);
            }
        }
        ok(())
    }
}

impl Stream for Packetize {
    type Item = Packet;
    type Error = &'static str;
    fn poll(&mut self) -> Result<Async<Option<Packet>>, &'static str> {
        self.rx.poll().map_err(|_| "websocket closed")
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
                Connecting(mut f, rx, d) => match f.poll() {
                    Ok(Async::NotReady) => {
                        self.state = Connecting(f, rx, d);
                        return Ok(Async::NotReady);
                    }
                    Ok(Async::Ready(conn)) => {
                        self.state = Handshake(
                            HandshakeProto::new(conn, SimpleAuthorizer::new(
                                "cantal.internal", "/graphql")),
                            rx, d);
                    }
                    Err(e) => {
                        error!("Error connecting to {}: {}", self.id, e);
                        return Err(());
                    }
                }
                Handshake(mut f, rx, d) => match f.poll() {
                    Ok(Async::NotReady) => {
                        self.state = Handshake(f, rx, d);
                        return Ok(Async::NotReady);
                    }
                    Ok(Async::Ready((out, inp, ()))) => {
                        d.tx.unbounded_send(Packet::Text(
                            serde_json::to_string(
                                &OutputMessage::ConnectionInit {
                                    payload: ConnectionParams {},
                                }
                            ).expect("serialize connection_init"),
                        )).ok();
                        self.state = Active(Loop::client(out, inp,
                            Packetize { rx },
                            d, &*CONFIG, &handle()));
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
    pub fn new(id: &Id, hostname: &Hostname, addr: SocketAddr, shared: &Shared)
        -> Connection
    {
        info!("Connecting to {}:{} via {}", hostname, id, addr);
        let (tx, rx) = unbounded();
        let disp = Dispatcher {
            tx,
            hostname: hostname.clone(),
            shared: shared.clone(),
        };
        Connection {
            id: id.clone(),
            shared: shared.clone(),
            state: State::Connecting(TcpStream::connect(&addr), rx, disp),
        }
    }
}
