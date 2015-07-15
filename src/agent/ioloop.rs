use std::collections::HashMap;

use mio;
use mio::{Token, Timeout, EventSet, Sender, Evented};

use super::deps::Dependencies;


trait HandleConn {
    fn ready(&mut self, &mut EventLoop<Handler>, tok: Token,
        ev: EventSet, deps: Dependencies) -> bool;
}

enum HttpTest {
    KeepAlive,
    Headers(Vec<u8>),
    Body(Vec<u8>),
    Response(Vec<u8>, Vec<u8>),
}

enum Websock

impl HandleConn for HttpTest {
    fn
}


struct Handler {
    dependencies: Dependencies,
    connections: HashMap<Token, Box<HandleConn>>,
}


impl mio::Handler for Handler {
    type Timeout = ();
    type Notify = ();

    fn ready(&mut self, event_loop: &mut EventLoop<Handler>,
        token: Token, events: EventSet) {
        if let Some(ref mut conn) = self.connections.get_mut(token) {
            if conn.ready(event_loop, token, events, self.deps) {
                return;
            }
        }
        self.connections.remove(token);
    }
}
