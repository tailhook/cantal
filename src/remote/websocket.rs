use std::collections::HashSet;
use std::fmt;
use std::hash::{Hasher};
use std::marker::PhantomData;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicUsize, AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use futures::{Future, Stream};
use futures::stream::MapErr;
use futures::future::{FutureResult, ok};
use futures::sync::mpsc::{UnboundedReceiver};
use serde_cbor::de::from_slice;
use tk_http::websocket::{self, Frame, Loop, ServerCodec};
use tk_easyloop::{spawn, handle};
use tk_bufstream::{WriteFramed, ReadFramed};
use tokio_io::{AsyncRead, AsyncWrite};

lazy_static! {
    static ref CONNECTION_ID: AtomicUsize = AtomicUsize::new(0);
}

#[derive(Clone)]
pub struct Connection(Arc<ConnectionState>);

struct ConnectionState {
    id: usize,
    addr: SocketAddr,
    connected: AtomicBool,
}


pub struct Dispatcher {
    connection: Connection,
}

impl Dispatcher {
    pub fn new(cli: Connection)
        -> Dispatcher
    {
        cli.0.connected.store(true, Ordering::SeqCst);
        let disp = Dispatcher {
            connection: cli.clone(),
        };
        return disp;
    }
}

impl Connection {

    pub fn is_connected(&self) -> bool {
        self.0.connected.load(Ordering::SeqCst)
    }

    pub fn addr(&self) -> SocketAddr {
        self.0.addr
    }

}

impl websocket::Dispatcher for Dispatcher {
    // TODO(tailhook) implement backpressure
    type Future = FutureResult<(), websocket::Error>;
    fn frame(&mut self, frame: &Frame) -> Self::Future {
        match *frame {
            Frame::Binary(data) => match from_slice(data) {
                Ok(()) => {
                    unimplemented!();
                }
                Err(e) => {
                    match *frame {
                        Frame::Binary(x) => {
                            error!("Failed to deserialize frame, \
                                error: {}, frame: {}", e,
                                String::from_utf8_lossy(x));
                        }
                        _ => {
                            error!("Failed to deserialize frame, \
                                error: {}, frame: {:?}", e, frame);
                        }
                    }
                }
            },
            _ => {
                error!("Bad frame received: {:?}", frame);
            }
        }
        ok(())
    }
}

impl ::std::hash::Hash for Connection {
    fn hash<H>(&self, state: &mut H)
        where H: Hasher
    {
        self.0.id.hash(state)
    }
}

impl PartialEq for Connection {
    fn eq(&self, other: &Connection) -> bool {
        self.0.id == other.0.id
    }
}

impl Eq for Connection {}
