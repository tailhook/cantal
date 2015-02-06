//  This main loop is written basically because mio is hard to compile at
//  the moment. We are going to drop it as soon as mio (or some other async io)
//  for rust (and may be std::io itself) are stable.
//
//  This is the reason we don't make it too generic, just get things done for
//  our case. I.e. we only support HTTP and epoll

use std::io::IoError;
use std::time::Duration;
use std::os::unix::Fd;
use std::collections::BinaryHeap;
use std::collections::HashMap;


pub mod http;


pub type HttpHandler = fn(req: &http::Request);
pub type IntervalHandler = fn();


enum Handler {
    Http(HttpHandler),
}

pub struct MainLoop {
    socket_handlers: HashMap<Fd, Handler>,
}

impl MainLoop {
    pub fn new() -> Result<MainLoop, IoError> {
        return Ok(MainLoop {
            socket_handlers: HashMap::new(),
        });
    }

    pub fn add_http_server(&mut self, host: &str, port: u16,
                           handler: HttpHandler)
        -> Result<(), IoError>
    {
        Ok(())
    }

    pub fn add_interval(&mut self, duration: Duration,
                        handler: IntervalHandler)
    {
    }

    pub fn run(&mut self) -> ! {
        unimplemented!();
    }
}
