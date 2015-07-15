use std::io;
use std::mem::replace;
use std::net::SocketAddr;

use httparse;
use mio::{EventLoop, Token, PollOpt, EventSet};
use mio::tcp::TcpStream;

use super::super::server::Handler;
use super::super::server::Context;
use super::super::websock;
use super::super::util::WriteVec as W;
use super::super::util::ReadVec as R;
use super::super::util::Consume;


const MAX_WEBSOCK_MESSAGE: usize = 1 << 20;


pub struct WebSocket {
    sock: TcpStream,
    pub handshake: bool,
    input: Vec<u8>,
    output: Vec<u8>,
}


impl WebSocket {
    pub fn connect(addr: SocketAddr) -> Result<WebSocket, io::Error>
    {
        Ok(WebSocket {
            sock: try!(TcpStream::connect(&addr)),
            handshake: true,
            input: Vec::new(),
            output: b"\
                GET /ws HTTP/1.1\r\n\
                Host: cantal\r\n\
                Upgrade: websocket\r\n\
                Connection: Upgrade\r\n\
                Sec-WebSocket-Version: 13\r\n\
                Sec-WebSocket-Key: x3JJHMbDL1EzLkh9GBhXDw==\r\n\
                \r\n".to_vec(),
        })
    }
    pub fn register(&self, tok: Token, eloop: &mut EventLoop<Handler>)
        -> Result<(), io::Error>
    {
        eloop.register_opt(&self.sock, tok,
            EventSet::writable(), PollOpt::level())
    }
    pub fn events(&mut self, ev: EventSet, tok: Token, ctx: &mut Context)
        -> bool
    {
        if ev.is_writable() {
            let buf = replace(&mut self.output, Vec::new());
            match W::write(&mut self.sock, buf) {
                W::Done => {
                    match ctx.eloop.reregister(&self.sock, tok,
                        EventSet::readable(), PollOpt::level())
                    {
                        Ok(_) => return true,
                        Err(e) => {
                            error!("Error on reregister: {}; \
                                    closing connection", e);
                        }
                    }
                }
                W::More(buf) => {
                    self.output = buf;
                }
                // Assume that read end will be closed too
                W::Close => {}
                W::Error(err) => {
                    debug!("Error writing to websock: {}", err);
                }
            }
        }
        if ev.is_readable() {
            match R::read(&mut self.sock, &mut self.input,
                          MAX_WEBSOCK_MESSAGE)
            {
                R::Wait => { return true; }
                R::More => {
                    if self.handshake {
                        let consumed = {
                            let mut headers = [httparse::EMPTY_HEADER; 64];
                            let mut resp = httparse::Response::new(&mut headers);
                            match resp.parse(&self.input) {
                                Ok(httparse::Status::Complete(len)) => {
                                    // TODO(tailhook) handshake validation
                                    len
                                }
                                Ok(httparse::Status::Partial) => {
                                    return true;
                                }
                                Err(err) => {
                                    debug!("Error while reading request: {:?}",
                                        err);
                                    ctx.eloop.deregister(&self.sock)
                                        .map_err(|err| error!(
                                            "Can't deregister sock: {}",
                                            err))
                                        .ok();
                                    return false;
                                }
                            }
                        };
                        self.input.consume(consumed);
                        self.handshake = false;
                    }
                    websock::parse_message(&mut self.input, ctx,
                        |opcode, msg, ctx| {
                            println!("Remote message {:?} {:?}", opcode,
                                ::std::str::from_utf8(msg));

                        });
                    return true;
                }
                R::Full|R::Close => {}
                R::Error(err) => {
                    debug!("Error reading from websock: {}", err);
                }
            }
            ctx.eloop.deregister(&self.sock)
                .map_err(|err| error!("Can't deregister sock: {}", err))
                .ok();
            return false;
        }
        return true;
    }
}

