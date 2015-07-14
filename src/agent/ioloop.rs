use std::collections::HashMap;

use mio;
use mio::{Token, Timeout, EventSet, Sender, Evented};

use super::deps::Dependencies;


enum Notify {
    DestroySocket(Box<Evented+Send>),
}

struct SocketRef<T:Evented+Send+'static>(Option<T>, Sender<Notify>);

impl<T:Evented+Send+'static> Drop for SocketRef<T> {
    fn drop(&mut self) {
        self.1.send(Notify::DestroySocket(
            Box::new(self.0.take().unwrap()) as Box<Evented+Send>));
    }
}

trait Acceptor {
    fn accept(&self, TcpStream) -> Option<Streamer>;
}

trait Streamer {
    fn write_buf<'x>(&'x mut self) -> Option<&mut Vec<u8>>;
    fn read_buf<'x>(&'x mut self) -> Option<&mut Vec<u8>>;
    fn process_data(self) -> Option<Self>;
}

enum Sink {
    Accept(Box<Acceptor>),
    Stream(Box<Streamer>),
}


struct Cell {
    event_set: EventSet,
    timeout: Option<(u64, Timeout)>,
    consumer: Box<Consumer>,
}


struct Handler {
    dependencies: Dependencies,
    connections: HashMap<Token, Cell>,
}


impl mio::Handler for Handler {
    fn ready(
}
