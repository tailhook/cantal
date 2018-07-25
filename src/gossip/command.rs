use std::net::SocketAddr;


#[derive(Debug)]
pub enum Command {
    AddHost(SocketAddr),
}

