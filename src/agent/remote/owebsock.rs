use std::io;
use std::mem::replace;
use std::net::SocketAddr;

use httparse;
use mio::{EventLoop, Token, PollOpt, EventSet};
use mio::tcp::TcpStream;
use probor;

use super::super::server::Handler;
use super::super::server::Context;
use super::super::websock;
use super::super::util::WriteVec as W;
use super::super::util::ReadVec as R;
use super::super::util::Consume;
use super::super::ioutil::Poll;
//use super::super::websock::InputMessage as OutputMessage;
use super::super::websock::OutputMessage as InputMessage;


const MAX_WEBSOCK_MESSAGE: usize = 1 << 20;


pub struct WebSocket {
    pub sock: TcpStream,
    pub handshake: bool,
    pub input: Vec<u8>,
    pub output: Vec<u8>,
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
        eloop.register(&self.sock, tok, EventSet::writable(), PollOpt::level())
    }
    pub fn events(&mut self, ev: EventSet, tok: Token, ctx: &mut Context)
        -> Option<Vec<InputMessage>>
    {
        if ev.is_writable() {
            let buf = replace(&mut self.output, Vec::new());
            match W::write(&mut self.sock, buf) {
                W::Done => {
                    ctx.eloop.modify(&self.sock, tok, true, false);
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
                R::Wait => { return Some(Vec::new()); }
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
                                    return Some(Vec::new());
                                }
                                Err(err) => {
                                    debug!("Error while reading request: {:?}",
                                        err);
                                    ctx.eloop.remove(&self.sock);
                                    return None;
                                }
                            }
                        };
                        self.input.consume(consumed);
                        self.handshake = false;
                    }
                    let mut messages: Vec<InputMessage> = vec!();
                    loop {
                        let msg: Option<InputMessage>;
                        msg = websock::parse_message(&mut self.input, ctx,
                            |opcode, msg, _ctx| {
                                if opcode == websock::Opcode::Binary {
                                    probor::from_slice(msg)
                                    .map_err(|e| error!(
                                        "Error decoding msg {:?}", e))
                                    .ok()
                                } else {
                                    debug!("Wrong opcode. Skipping");
                                    None
                                }
                            });
                        if let Some(msg) = msg {
                            messages.push(msg);
                        } else {
                            break;
                        }
                    }
                    return Some(messages);
                }
                R::Full => {
                    debug!("Input buffer is full (too big message), \
                        closing connection.");
                }
                R::Close => {}
                R::Error(err) => {
                    debug!("Error reading from websock: {}", err);
                }
            }
            ctx.eloop.remove(&self.sock);
            return None;
        }
        return Some(Vec::new());
    }
}

