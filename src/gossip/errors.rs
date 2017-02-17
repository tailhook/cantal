use std::io;
use std::net::{SocketAddr, AddrParseError};

quick_error! {
    #[derive(Debug)]
    pub enum InitError {
        Host(addr: String, e: AddrParseError) {
            context(addr: &'a str, e: AddrParseError) -> (addr.to_string(), e)
            context(addr: &'a String, e: AddrParseError)
                -> (addr.to_string(), e)
            display("Can't parse hostname {:?}: {}", addr, e)
        }
        Bind(addr: SocketAddr, e: io::Error) {
            context(addr: SocketAddr, e: io::Error) -> (addr, e)
            display("Can't bind addr {:?}: {}", addr, e)
        }
    }
}

