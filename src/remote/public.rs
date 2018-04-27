use std::sync::{Arc, Mutex};

use slab::Slab;


pub struct Incoming(Arc<Mutex<Conns>>);

pub struct Token {
    conns: Arc<Mutex<Conns>>,
    index: usize,
}

pub struct Connection {
}

pub struct Conns {
    slab: Slab<Connection>,
}

impl Drop for Token {
    fn drop(&mut self) {
        self.conns.lock().expect("conns by token").slab.remove(self.index);
    }
}
