//  This main loop is written basically because mio is hard to compile at
//  the moment. We are going to drop it as soon as mio (or some other async io)
//  for rust (and may be std::io itself) are stable.
//
//  This is the reason we don't make it too generic, just get things done for
//  our case. I.e. we only support HTTP and epoll

use std::rc::Rc;
use std::io::Error;
use std::time::Duration;
use std::fmt::{Show, Formatter};
use std::fmt::Error as FmtError;
use std::os::unix::Fd;
use std::collections::BinaryHeap;
use std::collections::HashMap;

use self::lowlevel::WriteResult;
use self::HandlerResult::*;
use self::ReadHandler::*;
use self::WriteHandler::*;


pub mod http;
pub mod lowlevel;


pub type HttpHandler<'a> = &'a (for<'b> Fn(&'b http::Request<'b>)
                       -> Result<http::Response, http::Error> + 'a);
pub type IntervalHandler = fn();

enum ReadHandler<'a> {
    AcceptHttp(HttpHandler<'a>),
    ParseHttp(Box<http::Stream<'a>>),
}
enum WriteHandler {
    TailChunk(usize, Vec<u8>),
}

pub enum HandlerResult {
    ContinueRead,
    Close,
    SendAndClose(Vec<u8>),
}


pub struct MainLoop<'a> {
    epoll: lowlevel::EPoll,
    read_handlers: HashMap<Fd, ReadHandler<'a>>,
    write_handlers: HashMap<Fd, WriteHandler>,
}

impl<'a> MainLoop<'a> {
    pub fn new<'x>() -> Result<MainLoop<'x>, Error> {
        return Ok(MainLoop {
            epoll: try!(lowlevel::EPoll::new()),
            read_handlers: HashMap::new(),
            write_handlers: HashMap::new(),
        });
    }

    pub fn add_http_server(&mut self, host: &str, port: u16,
        handler: HttpHandler<'a>)
        -> Result<(), Error>
    {
        let fd = try!(lowlevel::bind_tcp_socket(host, port));
        assert!(self.read_handlers.insert(
            fd, ReadHandler::AcceptHttp(handler)).is_none());
        self.epoll.add_fd_in(fd);
        Ok(())
    }

    pub fn add_interval(&mut self, duration: Duration,
                        handler: IntervalHandler)
    {
    }

    pub fn run(&'a mut self) -> ! {
        loop {
            match self.epoll.next_event(None) {
                lowlevel::EPollEvent::Input(fd) => {
                    enum R<'a> {
                        AddHttp(Fd, HttpHandler<'a>),
                        SwitchToSend(Fd, Vec<u8>),
                        Remove(Fd),
                        Proceed,
                    }
                    let res = {
                        let handle = self.read_handlers.get_mut(&fd)
                            .expect("Bad file descriptor returned from epoll");
                        match handle {
                            &mut AcceptHttp(hdl) => {
                                lowlevel::accept(fd)
                                .map_err(|e| info!("Error accepting: {}", e))
                                .map(|sock| R::AddHttp(sock, hdl))
                                .unwrap_or(R::Proceed)
                            }
                            &mut ParseHttp(ref mut stream) => {
                                match stream.read_http() {
                                    ContinueRead => R::Proceed,
                                    Close => R::Remove(stream.fd),
                                    SendAndClose(data) => {
                                        // TODO(tailhook) send right now
                                        R::SwitchToSend(fd, data)
                                    }
                                }
                            }
                        }
                    };
                    match res {
                        R::AddHttp(sock, hdl) => {
                            assert!(self.read_handlers.insert(sock,
                                ParseHttp(
                                    Box::new(http::Stream::new(sock, hdl))),
                                ).is_none());
                            self.epoll.add_fd_in(sock);
                        }
                        R::SwitchToSend(sock, data) => {
                            self.epoll.change_to_out(sock);
                            assert!(self.read_handlers.remove(&sock)
                                    .is_some());
                            assert!(self.write_handlers.insert(sock,
                                TailChunk(0, data),
                                ).is_none());
                        }
                        R::Remove(sock) => {
                            self.epoll.del_fd(sock);
                            assert!(self.read_handlers.remove(&sock)
                                    .is_some());
                            lowlevel::close(sock);
                        }
                        R::Proceed => {}
                    }
                }
                lowlevel::EPollEvent::Output(fd) => {
                    enum R {
                        Remove(Fd),
                        Proceed,
                    }
                    let res = {
                        let handle = self.write_handlers.get_mut(&fd)
                            .expect("Bad file descriptor returned from epoll");
                        match handle {
                            &mut TailChunk(ref mut offset, ref val) => {
                                match lowlevel::write(fd, &val[*offset..]) {
                                    WriteResult::Written(bytes) => {
                                        *offset += bytes;
                                        if *offset >= val.len() {
                                            R::Remove(fd)
                                        } else {
                                            R::Proceed
                                        }
                                    }
                                    WriteResult::Fatal(err) => {
                                        error!("Error handling connection\
                                            (fd: {}): {:?}", fd, err);
                                        R::Remove(fd)
                                    }
                                    WriteResult::Again => {
                                        R::Proceed
                                    }
                                    WriteResult::Closed => {
                                        R::Remove(fd)
                                    }
                                }
                            }
                        }
                    };
                    match res {
                        R::Remove(sock) => {
                            self.epoll.del_fd(sock);
                            assert!(self.write_handlers.remove(&sock)
                                    .is_some());
                            lowlevel::close(sock);
                        }
                        R::Proceed => {}
                    }
                }
                lowlevel::EPollEvent::Timeout => {
                    unimplemented!();
                }
            }
        }
    }
}
