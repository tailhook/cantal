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
pub mod lowlevel;


pub type HttpHandler = fn(req: &http::Request);
pub type IntervalHandler = fn();

enum SockHandler {
    AcceptHttp(HttpHandler),
    ParseHttp(HttpHandler),
}

pub struct MainLoop {
    epoll: lowlevel::EPoll,
    socket_handlers: HashMap<Fd, SockHandler>,
}

impl MainLoop {
    pub fn new() -> Result<MainLoop, IoError> {
        return Ok(MainLoop {
            epoll: try!(lowlevel::EPoll::new()),
            socket_handlers: HashMap::new(),
        });
    }

    pub fn add_http_server(&mut self, host: &str, port: u16,
                           handler: HttpHandler)
        -> Result<(), IoError>
    {
        let fd = try!(lowlevel::bind_tcp_socket(host, port));
        self.socket_handlers.insert(fd, SockHandler::AcceptHttp(handler));
        self.epoll.add_fd_in(fd);
        Ok(())
    }

    pub fn add_interval(&mut self, duration: Duration,
                        handler: IntervalHandler)
    {
    }

    pub fn run(&mut self) -> ! {
        match self.epoll.next_event(None) {
            lowlevel::EPollEvent::Input(fd) => {
                println!("INPUT FD {}", fd);
                match self.socket_handlers.get(&fd)
                    .expect("Unexpected file descriptor returned from epoll")
                {
                    &SockHandler::AcceptHttp(ref hdl) => {
                        unimplemented!();
                    }
                    &SockHandler::ParseHttp(ref hdl) => {
                        unimplemented!();
                    }
                }
            }
            lowlevel::EPollEvent::Timeout => {
                unimplemented!();
            }
        }
    }
}
