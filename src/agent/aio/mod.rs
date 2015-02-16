//  This main loop is written basically because mio is hard to compile at
//  the moment. We are going to drop it as soon as mio (or some other async io)
//  for rust (and may be std::io itself) are stable.
//
//  This is the reason we don't make it too generic, just get things done for
//  our case. I.e. we only support HTTP and epoll

use std::io::IoError;
use std::time::Duration;
use std::fmt::{Show, Formatter};
use std::fmt::Error as FmtError;
use std::os::unix::Fd;
use std::collections::BinaryHeap;
use std::collections::HashMap;

use self::HandlerResult::*;


pub mod http;
pub mod lowlevel;


pub type HttpHandler = fn(req: &http::Request);
pub type IntervalHandler = fn();

enum SockHandler {
    AcceptHttp(HttpHandler),
    ParseHttp(Box<http::Stream>),
}

pub enum HandlerResult {
    AddHttp(Fd, HttpHandler),
    Remove(Fd),
    Proceed,
}

impl Show for SockHandler {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), FmtError> {
        match *self {
            SockHandler::AcceptHttp(ref hdl) => {
                write!(fmt, "AcceptHttp(_)")
            }
            SockHandler::ParseHttp(ref hdl) => {
                write!(fmt, "ParseHttp(_)")
            }
        }
    }
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
        loop {
            match self.epoll.next_event(None) {
                lowlevel::EPollEvent::Input(fd) => {
                    let res = {
                        let handle = self.socket_handlers.get_mut(&fd)
                            .expect("Bad file descriptor returned from epoll");
                        match handle {
                            &mut SockHandler::AcceptHttp(hdl) => {
                                lowlevel::accept(fd)
                                .map_err(|e| info!("Error accepting: {}", e))
                                .map(|sock| AddHttp(sock, hdl))
                                .unwrap_or(Proceed)
                            }
                            &mut SockHandler::ParseHttp(ref mut stream) => {
                                stream.read_http()
                            }
                        }
                    };
                    match res {
                        AddHttp(sock, hdl) => {
                            self.socket_handlers.insert(sock,
                                SockHandler::ParseHttp(
                                    Box::new(http::Stream::new(sock, hdl))));
                            self.epoll.add_fd_in(sock);
                        }
                        Remove(sock) => {
                            self.epoll.del_fd(sock);
                            assert!(self.socket_handlers.remove(&sock)
                                    .is_some());
                            lowlevel::close(sock);
                        }
                        Proceed => {}
                    }
                }
                lowlevel::EPollEvent::Timeout => {
                    unimplemented!();
                }
            }
        }
    }
}
