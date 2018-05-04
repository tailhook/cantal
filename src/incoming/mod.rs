use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::borrow::Borrow;

use futures::sync::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use futures::Stream;
use futures::stream::MapErr;
use tk_bufstream::{WriteFramed, ReadFramed};
use tk_easyloop::handle;
use tk_http::websocket::{Loop, ServerCodec, Config, Packet};
use tokio_io::{AsyncRead, AsyncWrite};

use frontend::graphql;

mod dispatcher;

lazy_static! {
    static ref WEBSOCK_CONFIG: Arc<Config> = Config::new()
        .done();
    static ref CONNECTION_ID: AtomicUsize = AtomicUsize::new(0);
}

// TODO(tailhook) change to u64 when atomics are fixed
#[derive(Debug, Eq, PartialEq, Clone, Copy, Hash)]
struct Id(usize);

pub struct Token {
    holder: Incoming,
    key: Id,
}

#[derive(Debug, Clone)]
pub struct Connection {
    id: Id,
    tx: UnboundedSender<Packet>,
}

#[derive(Debug, Clone)]
pub struct Incoming(Arc<Mutex<Internal>>);

#[derive(Debug)]
struct Internal {
    connections: HashSet<Connection>,
}

fn closed(():()) -> &'static str {
    "channel closed"
}

impl Incoming {
     pub fn new() -> Incoming {
        Incoming(Arc::new(Mutex::new(Internal {
            connections: HashSet::new(),
        })))
     }
     pub fn connected<S>(&self,
               output: WriteFramed<S, ServerCodec>,
               input: ReadFramed<S, ServerCodec>,
               graphql: &graphql::Context)
        -> (Token, Loop<S,
            MapErr<UnboundedReceiver<Packet>, fn(()) -> &'static str>,
            dispatcher::Dispatcher>)
        where S: AsyncRead + AsyncWrite,
     {
        let (tx, rx) = unbounded();
        let rx = rx.map_err(closed as fn(()) -> &'static str);
        let id = Id(CONNECTION_ID.fetch_add(1, Ordering::Relaxed));
        let conn = Connection { id, tx };
        let disp = dispatcher::Dispatcher {
            conn: conn.clone(),
            graphql: graphql.clone(),
        };
        self.0.lock().expect("lock works")
            .connections.insert(conn);
        let fut = Loop::server(output, input, rx, disp,
            &*WEBSOCK_CONFIG, &handle());
        return (Token {
            key: id,
            holder: self.clone(),
        }, fut);
     }
}

impl Borrow<Id> for Connection {
    fn borrow(&self) -> &Id {
        &self.id
    }
}

impl Drop for Token {
    fn drop(&mut self) {
        self.holder.0.lock().expect("lock works")
            .connections.remove(&self.key);
    }
}

impl PartialEq for Connection {
    fn eq(&self, other: &Connection) -> bool {
        self.id == other.id
    }
}
impl Eq for Connection {}

impl ::std::hash::Hash for Connection {
    fn hash<H: ::std::hash::Hasher>(&self, h: &mut H)  {
        self.id.hash(h)
    }
}
