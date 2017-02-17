use std::net::SocketAddr;


pub enum Command {
    AddHost(SocketAddr),
}

