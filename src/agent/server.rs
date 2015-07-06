use std::str::from_utf8;
use std::io::{Read, Write};
use std::io::ErrorKind::{Interrupted, WouldBlock};
use std::fmt::{Debug, Formatter};
use std::fmt::Error as FmtError;
use std::net::{SocketAddr, SocketAddrV4};
use std::sync::RwLock;
use std::sync::atomic::{AtomicUsize, Ordering, ATOMIC_USIZE_INIT};
use std::default::Default;
use std::collections::HashMap;

use httparse;
use rustc_serialize::json;
use rustc_serialize::json::ToJson;
use mio;
use hyper::buffer::BufReader;
use mio::buf::{RingBuf, ByteBuf, MutByteBuf, MutBuf, Buf};
use mio::{TryRead, Interest, PollOpt};
use mio::{EventLoop, Token, ReadHint, Handler};
use hyper::uri::RequestUri;
use hyper::header::{Headers, ContentLength};
use hyper::method::Method;
use hyper::version::HttpVersion as Version;
use hyper::http::h1::parse_request;
use hyper::error::Error as HyperError;

use super::scan;
use super::http;
use super::staticfiles;
use super::stats::{Stats};
use super::storage::{StorageStats};
use super::rules::{Query, query};
use super::p2p::Command;
use super::http::NotFound;
use super::http::Request;


const INPUT: Token = Token(0);
const MAX_HEADER_SIZE: usize = 8192;
static TOKEN_COUNTER: AtomicUsize = ATOMIC_USIZE_INIT;

type Loop<'x> = &'x mut EventLoop<Context<'x>>;

#[derive(RustcEncodable)]
struct StatusData {
    pub startup_time: u64,
    pub scan_duration: u32,
    pub storage: StorageStats,
    pub boot_time: Option<u64>,
}

#[derive(RustcEncodable)]
struct ProcessesData<'a> {
    boot_time: Option<u64>,
    all: &'a Vec<scan::processes::MinimalProcess>,
}

/*
fn handle_request(stats: &RwLock<Stats>, req: &http::Request,
    gossip_cmd: mio::Sender<Command>)
    -> Result<http::Response, http::Error>
{
    if  req.uri().starts_with("/js") ||
        req.uri().starts_with("/css/") ||
        req.uri().starts_with("/fonts/") ||
        req.uri() == "/"
    {
        return staticfiles::serve(req);
    } else {
        let stats = stats.read().unwrap();
        let ref h = stats.history;
        match req.uri() {
            "/status.json" => Ok(http::reply_json(req, &StatusData {
                startup_time: stats.startup_time,
                scan_duration: stats.scan_duration,
                storage: stats.storage,
                boot_time: stats.boot_time,
            })),
            "/all_processes.json" => Ok(http::reply_json(req, &ProcessesData {
                boot_time: stats.boot_time,
                all: &stats.processes,
            })),
            "/all_metrics.json" => Ok(http::reply_json(req,
                &stats.history.tip.keys()
                .chain(stats.history.fine.keys())
                .chain(stats.history.coarse.keys())
                .collect::<Vec<_>>()
                .to_json()
            )),
            "/all_peers.json" => Ok(http::reply_json(req,
                &json::Json::Object(vec![
                    (String::from("peers"), json::Json::Array(
                        stats.gossip.read().unwrap().peers.values()
                        .map(ToJson::to_json)
                        .collect())),
                ].into_iter().collect()
            ))),
            "/query.json"
            => from_utf8(req.body.unwrap_or(b""))
               .map_err(|_| http::Error::BadRequest("Bad utf-8 encoding"))
               .and_then(|s| json::decode::<Query>(s)
               .map_err(|_| http::Error::BadRequest("Failed to decode query")))
               .and_then(|r| {
                   Ok(http::reply_json(req, &vec![
                    (String::from("dataset"), try!(query(&r, &*stats))),
                    (String::from("tip_timestamp"), h.tip_timestamp.to_json()),
                    (String::from("fine_timestamps"), h.fine_timestamps
                        .iter().cloned().collect::<Vec<_>>().to_json()),
                    (String::from("coarse_timestamps"), h.coarse_timestamps
                        .iter().cloned().collect::<Vec<_>>().to_json()),
                   ].into_iter().collect::<HashMap<_,_>>().to_json()))
                }),
            "/add_host.json" => {
                #[derive(RustcDecodable)]
                struct Query {
                    addr: String,
                }
                from_utf8(req.body.unwrap_or(b""))
               .map_err(|_| http::Error::BadRequest("Bad utf-8 encoding"))
               .and_then(|x| json::decode(x)
               .map_err(|e| error!("Error parsing query: {:?}", e))
               .map_err(|_| http::Error::ServerError("Request format error")))
               .and_then(|x: Query| x.addr.parse()
               .map_err(|_| http::Error::BadRequest("Can't parse IP address")))
               .and_then(|x| gossip_cmd.send(Command::AddGossipHost(x))
               .map_err(|e| error!("Error sending to p2p loop: {:?}", e))
               .map_err(|_| http::Error::ServerError("Notify Error")))
               .and_then(|_| {
                    Ok(http::reply_json(req, &vec![
                        (String::from("ok"), true)
                    ].into_iter().collect::<HashMap<_, _>>().to_json()))
                })
            }
            _ => Err(http::Error::NotFound),
        }
    }
}
*/

fn new_token() -> Token {
    Token(TOKEN_COUNTER.fetch_add(1, Ordering::Relaxed))
}

fn make_request(req: httparse::Request) -> Result<Request, HyperError> {
    Ok(Request {
        version: if req.version.unwrap() == 1
            { Version::Http11 } else { Version::Http10 },
        method: try!(req.method.unwrap().parse()),
        uri: try!(req.path.unwrap().parse()),
        headers: try!(Headers::from_raw(req.headers)),
        body: Some(vec!()),
    })
}

enum Mode {
    ReadHeaders,
    ReadContent(MutByteBuf),
    WriteResponse(Vec<u8>, usize),
}

impl Debug for Mode {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), FmtError> {
        use self::Mode::*;
        match self {
            &ReadHeaders => write!(fmt, "ReadHeaders"),
            &ReadContent(ref x) => write!(fmt, "ReadContent({}/{})",
                x.remaining(), x.capacity()),
            &WriteResponse(ref x, y) => write!(fmt, "WriteResponse({}/{})",
                y, x.len()),
        }
    }
}

#[derive(Debug)]
struct HttpClient {
    token: Token,
    sock: mio::tcp::TcpStream,
    buf: RingBuf,
    mode: Mode,
}

struct Context<'a> {
    input: mio::tcp::TcpListener,
    stats: &'a RwLock<Stats>,
    http_inputs: HashMap<Token, HttpClient>,
}

impl HttpClient {
    fn resolve(&self, req: &Request)
        -> Result<http::Response, Box<http::Error>>
    {
        use hyper::method::Method::*;
        use hyper::uri::RequestUri::AbsolutePath as P;
        match (&req.method, &req.uri) {
            (&Get, &P(ref x)) if &x[..] == "/"
            => staticfiles::serve(req),
            (&Get, &P(ref x)) if x.starts_with("/js/")
            => staticfiles::serve(req),
            (&Get, &P(ref x)) if x.starts_with("/css/")
            => staticfiles::serve(req),
            (&Get, &P(ref x)) if x.starts_with("/fonts/")
            => staticfiles::serve(req),
            //(Get, P("/status.json")) => self.serve_status(req),
            //(Get, P("/all_processes.json")) => self.serve_processes(req),
            //(Get, P("/all_metrics.json")) => self.serve_metrics(req),
            //(Get, P("/all_peers.json")) => self.serve_peers(req),
            //(Post, P("/query.json")) => self.serve_query(req),
            //(Post, P("/add_host.json")) => self.do_add_host(req),
            _ => Err(Box::new(NotFound) as Box<http::Error>),
        }
    }
    fn write(&mut self, eloop: &mut EventLoop<Context>) -> bool {
        use self::Mode::*;
        match self.mode {
            ReadHeaders => unreachable!(),
            ReadContent(_) => unreachable!(),
            WriteResponse(ref buf, ref mut off) => {
                match self.sock.write(&buf[*off..]) {
                    Ok(0) => {
                        return false;
                    }
                    Ok(x) => {
                        *off += x;
                        if *off >= buf.len() {
                            // TODO(tailhook) arrange for next request
                            return false;
                        }
                        return true;
                    }
                    Err(ref x) if x.kind() == Interrupted ||
                              x.kind() == WouldBlock
                    => {
                        return true;
                    }
                    Err(x) => {
                        error!("Error writing response: {:?}", x);
                        return false;
                    }
                }
            }
        }
    }
    fn read(&mut self, eloop: &mut EventLoop<Context>) -> bool {
        use self::Mode::*;
        match self.mode {
            ReadHeaders => self.read_headers(eloop),
            ReadContent(_) => unimplemented!(),
            WriteResponse(_, _) => unreachable!(),
        }
    }
    fn read_headers(&mut self, eloop: &mut EventLoop<Context>) -> bool {
        debug!("Http {:?}", self);
        if(MutBuf::remaining(&self.buf) == 0) {
            debug!("Headers too long");
            return false;
        }
        let res = self.sock.try_read_buf(&mut self.buf);
        match res {
            Ok(Some(0)) => {
                return false; // Connection closed
            }
            Ok(Some(x)) => {
                use hyper::error::Error::*;
                let mut headers = [httparse::EMPTY_HEADER; 64];
                let (rreq, len) = {
                    let mut req = httparse::Request::new(&mut headers);
                    let len = match req.parse(Buf::bytes(&self.buf)) {
                        Ok(httparse::Status::Complete(len)) => len,
                        Ok(httparse::Status::Partial) => {
                            trace!("Partial read");
                            return true;
                        }
                        Err(err) => {
                            debug!("Error while reading request: {:?}", err);
                            // TODO(tailhook) respond with bad request
                            return false;
                        }
                    };
                    let rreq = match make_request(req) {
                        Ok(rreq) => rreq,
                        Err(e) => {
                            debug!("Error decoding request: {:?}", e);
                            // TODO(tailhook) respond with bad request
                            return false;
                        }
                    };
                    (rreq, len)
                };
                MutBuf::advance(&mut self.buf, len);
                self.buf.mark(); // TODO(tailhook) needed ?
                match rreq.headers.get::<ContentLength>() {
                    Some(&ContentLength(x)) => {
                        self.mode = Mode::ReadContent(
                            ByteBuf::mut_with_capacity(x as usize));
                        return true;
                    }
                    None => {}
                }
                debug!("Got request, rreq {:?}", rreq);
                match self.resolve(&rreq) {
                    Ok(mut resp) => {
                        trace!("Response {:?}", resp);
                        self.mode = Mode::WriteResponse(
                            resp.to_buf(rreq.version), 0);
                    }
                    Err(err) => {
                        trace!("Error {:?}", err);
                        self.mode = Mode::WriteResponse(
                            err.to_response().to_buf(rreq.version), 0);
                    }
                }
                match eloop.reregister(&self.sock, self.token,
                    Interest::writable(), PollOpt::level())
                {
                    Ok(_) => {}
                    Err(e) => {
                        error!("Can't reregister http socket");
                        return false;
                    }
                }
                return true;
            }
            Ok(None) => {
                return true;
            }
            Err(e) => {
                error!("Error reading request {}", e);
                return false;
            }
        }
        return true;
    }
}

impl<'a> Handler for Context<'a> {
    type Timeout = ();
    type Message = ();

    fn readable(&mut self, eloop: &mut EventLoop<Context>,
                tok: Token, _hint: ReadHint)
    {
        match tok {
            INPUT => {
                let sock = match self.input.accept() {
                    Ok(Some(sock)) => sock,
                    Ok(None) => return,
                    Err(e) => {
                        error!("Can't accept connection: {}", e);
                        return;
                    }
                };
                debug!("Accepted {:?}", sock.peer_addr());
                let tok = new_token();
                if let Err(e) = eloop.register_opt(&sock, tok,
                    Interest::readable(), PollOpt::level()) {
                    error!("Error registering accepted connection: {}", e);
                }
                self.http_inputs.insert(tok, HttpClient {
                    token: tok,
                    sock: sock,
                    buf: RingBuf::new(MAX_HEADER_SIZE),
                    mode: Mode::ReadHeaders,
                }).ok_or(()).err().expect("Duplicate token in http_inputs");
            }
            tok => {
                let keep = match self.http_inputs.get_mut(&tok) {
                    Some(cli) => {
                        if !cli.read(eloop) {
                            eloop.deregister(&cli.sock);
                            false
                        } else {
                            true
                        }
                    }
                    None => {
                        error!("Unexpected token {:?}", tok);
                        return;
                    }
                };
                if !keep {
                    self.http_inputs.remove(&tok);
                }
            }
        }
    }

    fn writable(&mut self, eloop: &mut EventLoop<Context>, tok: Token)
    {
        match tok {
            INPUT => unreachable!(),
            tok => {
                let keep = match self.http_inputs.get_mut(&tok) {
                    Some(cli) => {
                        if !cli.write(eloop) {
                            eloop.deregister(&cli.sock);
                            false
                        } else {
                            true
                        }
                    }
                    None => {
                        error!("Unexpected token {:?}", tok);
                        return;
                    }
                };
                if !keep {
                    self.http_inputs.remove(&tok);
                }
            }
        }
    }
}

pub fn run_server(stats: &RwLock<Stats>, host: &str, port: u16,
    gossip_cmd: mio::Sender<Command>)
    -> Result<(), String>
{
    TOKEN_COUNTER.fetch_add(1, Ordering::Relaxed);  // start from 1
    let server = mio::tcp::TcpListener::bind(&SocketAddr::V4(
        SocketAddrV4::new(host.parse().unwrap(), port))).unwrap();
    let mut eloop = EventLoop::new().unwrap();
    eloop.register(&server, INPUT).unwrap();
    let mut ctx = Context {
        input: server,
        stats: stats,
        http_inputs: Default::default(),
    };
    eloop.run(&mut ctx)
        .map_err(|e| format!("Error running http loop: {}", e))
}
