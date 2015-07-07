use std::str::from_utf8;
use std::mem::swap;
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
use mio::{Interest, PollOpt};
use mio::{EventLoop, Token, ReadHint};
use unicase::UniCase;
use hyper::header::{Headers, ContentLength, Upgrade, ProtocolName};
use hyper::header::{Connection};
use hyper::header::ConnectionOption::ConnectionHeader;
use hyper::version::HttpVersion as Version;
use hyper::error::Error as HyperError;
use websocket::header::{WebSocketVersion, WebSocketKey};

use super::scan;
use super::http;
use super::staticfiles;
use super::stats::{Stats};
use super::storage::{StorageStats};
use super::rules::{Query, query};
use super::p2p::Command;
use super::http::{NotFound, BadRequest, ServerError, MethodNotAllowed};
use super::http::Request;
use super::util::ReadVec as R;


const INPUT: Token = Token(0);
const MAX_HEADER_SIZE: usize = 16384;
static TOKEN_COUNTER: AtomicUsize = ATOMIC_USIZE_INIT;

type Loop<'x> = &'x mut EventLoop<Handler<'x>>;

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
        body: Vec::new(),
    })
}

enum Mode {
    ReadHeaders,
    ReadContent(Request, Vec<u8>, usize),
    WriteResponse(Vec<u8>, usize),
}

impl Debug for Mode {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), FmtError> {
        use self::Mode::*;
        match self {
            &ReadHeaders => write!(fmt, "ReadHeaders"),
            &ReadContent(_, ref x, c) => write!(fmt, "ReadContent({}/{})",
                x.len(), c),
            &WriteResponse(ref x, y) => write!(fmt, "WriteResponse({}/{})",
                y, x.len()),
        }
    }
}

#[derive(Debug)]
struct HttpClient {
    token: Token,
    sock: mio::tcp::TcpStream,
    buf: Vec<u8>,
    mode: Mode,
}

struct Handler<'a> {
    input: mio::tcp::TcpListener,
    context: Context<'a>,
    http_inputs: HashMap<Token, HttpClient>,
}

struct Context<'a> {
    stats: &'a RwLock<Stats>,
    gossip_cmd: mio::Sender<Command>,
}

impl HttpClient {
    fn resolve(&mut self, req: &Request, context: &Context)
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
            (&Get, &P(ref x)) if &x[..] == "/ws"
            => self.respond_websock(req, context),
            (&Get, &P(ref x)) if &x[..] == "/status.json"
            => self.serve_status(req, context),
            (&Get, &P(ref x)) if &x[..] == "/all_processes.json"
            => self.serve_processes(req, context),
            (&Get, &P(ref x)) if &x[..] == "/all_metrics.json"
            => self.serve_metrics(req, context),
            (&Get, &P(ref x)) if &x[..] == "/all_peers.json"
            => self.serve_peers(req, context),
            (&Post, &P(ref x)) if &x[..] == "/query.json"
            => self.serve_query(req, context),
            (&Post, &P(ref x)) if &x[..] == "/add_host.json"
            => self.do_add_host(req, context),
            (&Get, _) => Err(Box::new(NotFound) as Box<http::Error>),
            _ => Err(Box::new(MethodNotAllowed) as Box<http::Error>),
        }
    }
    fn serve_status(&self, _req: &Request, context: &Context)
        -> Result<http::Response, Box<http::Error>>
    {
        let stats = context.stats.read().unwrap();
        Ok(http::Response::json(&StatusData {
                startup_time: stats.startup_time,
                scan_duration: stats.scan_duration,
                storage: stats.storage,
                boot_time: stats.boot_time,
            }))
    }
    fn serve_processes(&self, _req: &Request, context: &Context)
        -> Result<http::Response, Box<http::Error>>
    {
        let stats = context.stats.read().unwrap();
        Ok(http::Response::json(&ProcessesData {
                boot_time: stats.boot_time,
                all: &stats.processes,
            }))
    }
    fn serve_metrics(&self, _req: &Request, context: &Context)
        -> Result<http::Response, Box<http::Error>>
    {
        let stats = context.stats.read().unwrap();
        Ok(http::Response::json(
                &stats.history.tip.keys()
                .chain(stats.history.fine.keys())
                .chain(stats.history.coarse.keys())
                .collect::<Vec<_>>()
                .to_json()
            ))
    }
    fn serve_peers(&self, _req: &Request, context: &Context)
        -> Result<http::Response, Box<http::Error>>
    {
        let stats = context.stats.read().unwrap();
        let resp = http::Response::json(
            &json::Json::Object(vec![
                (String::from("peers"), json::Json::Array(
                    stats.gossip.read().unwrap().peers.values()
                    .map(ToJson::to_json)
                    .collect())),
            ].into_iter().collect()
           ));
        Ok(resp)
    }
    fn serve_query(&self, req: &Request, context: &Context)
        -> Result<http::Response, Box<http::Error>>
    {
        let stats = context.stats.read().unwrap();
        let h = &stats.history;
        from_utf8(&req.body)
           .map_err(|_| BadRequest::err("Bad utf-8 encoding"))
           .and_then(|s| json::decode::<Query>(s)
           .map_err(|_| BadRequest::err("Failed to decode query")))
           .and_then(|r| {
               Ok(http::Response::json(&vec![
                (String::from("dataset"), try!(query(&r, &*stats))),
                (String::from("tip_timestamp"), h.tip_timestamp.to_json()),
                (String::from("fine_timestamps"), h.fine_timestamps
                    .iter().cloned().collect::<Vec<_>>().to_json()),
                (String::from("coarse_timestamps"), h.coarse_timestamps
                    .iter().cloned().collect::<Vec<_>>().to_json()),
               ].into_iter().collect::<HashMap<_,_>>().to_json()))
            })
    }
    fn do_add_host(&self, req: &Request, context: &Context)
        -> Result<http::Response, Box<http::Error>>
    {
        #[derive(RustcDecodable)]
        struct Query {
            addr: String,
        }
        from_utf8(&req.body)
       .map_err(|_| BadRequest::err("Bad utf-8 encoding"))
       .and_then(|x| json::decode(x)
       .map_err(|e| error!("Error parsing query: {:?}", e))
       .map_err(|_| BadRequest::err("Request format error")))
       .and_then(|x: Query| x.addr.parse()
       .map_err(|_| BadRequest::err("Can't parse IP address")))
       .and_then(|x| context.gossip_cmd.send(Command::AddGossipHost(x))
       .map_err(|e| error!("Error sending to p2p loop: {:?}", e))
       .map_err(|_| ServerError::err("Notify Error")))
       .and_then(|_| {
            Ok(http::Response::json(&vec![
                (String::from("ok"), true)
            ].into_iter().collect::<HashMap<_, _>>().to_json()))
        })
    }
    fn respond_websock(&mut self, req: &Request, _context: &Context)
        -> Result<http::Response, Box<http::Error>>
    {
		if req.version != Version::Http11 {
			return Err(BadRequest::err("Unsupported request HTTP version"));
		}

		if req.headers.get() != Some(&(WebSocketVersion::WebSocket13)) {
			return Err(BadRequest::err("Unsupported WebSocket version"));
		}

		let key  = match req.headers.get::<WebSocketKey>() {
            Some(key) => key,
            None => {
                return Err(BadRequest::err("Missing Sec-WebSocket-Key"));
            }
		};

		match req.headers.get() {
			Some(&Upgrade(ref upgrade)) => {
				let mut correct_upgrade = false;
				for u in upgrade {
					if u.name == ProtocolName::WebSocket {
						correct_upgrade = true;
					}
				}
				if !correct_upgrade {
                    return Err(BadRequest::err(
                        "Invalid Upgrade WebSocket header"));
				}
			}
			None => {
                return Err(BadRequest::err("Missing Upgrade header"));
            }
		};

		match req.headers.get() {
			Some(&Connection(ref connection)) => {
				if !connection.contains(&(ConnectionHeader(
                    UniCase("Upgrade".to_string()))))
                {
                    return Err(BadRequest::err(
                        "Invalid Connection WebSocket header"));
				}
			}
			None => {
                return Err(BadRequest::err(
                    "Missing Connection WebSocket header"));
            }
		}

        Ok(http::Response::accept_websock(key))
    }
    fn write(&mut self, _eloop: &mut EventLoop<Handler>) -> bool {
        use self::Mode::*;
        match self.mode {
            ReadHeaders => unreachable!(),
            ReadContent(_, _, _) => unreachable!(),
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
    fn read(&mut self, eloop: &mut EventLoop<Handler>, context: &Context)
        -> bool
    {
        use self::Mode::*;
        match self.mode {
            ReadHeaders => self.read_headers(eloop, context),
            ReadContent(_, _, _) => self.read_content(eloop, context),
            WriteResponse(_, _) => unreachable!(),
        }
    }
    fn read_content(&mut self, eloop: &mut EventLoop<Handler>,
        context: &Context)
        -> bool
    {
        use self::Mode::*;
        let mut mode = ReadHeaders;
        swap(&mut self.mode, &mut mode);
        let req = match mode {
            ReadContent(mut req, mut buf, bytes) => {
                match R::read(&mut self.sock, &mut buf, bytes) {
                    R::Full => {
                        swap(&mut req.body, &mut buf);
                        req
                    }
                    R::More|R::Wait => {
                        self.mode = ReadContent(req, buf, bytes);
                        return true;
                    }
                    R::Close => {
                        debug!("Connection closed while reading request body");
                        return false;
                    }
                    R::Error(e) => {
                        debug!("Error while reading request body: {:?}", e);
                        return false;
                    }
                }
            }
            _ => unreachable!(),
        };
        match self.resolve(&req, context) {
            Ok(mut resp) => {
                trace!("Response {:?}", resp);
                self.mode = Mode::WriteResponse(
                    resp.to_buf(req.version), 0);
            }
            Err(err) => {
                trace!("Error {:?}", err);
                self.mode = Mode::WriteResponse(
                    err.to_response().to_buf(req.version), 0);
            }
        }
        match eloop.reregister(&self.sock, self.token,
            Interest::readable(), PollOpt::level())
        {
            Ok(_) => {}
            Err(e) => {
                error!("Can't reregister http socket: {}", e);
                return false;
            }
        }
        return true;
    }
    fn read_headers(&mut self, eloop: &mut EventLoop<Handler>,
        context: &Context)
        -> bool
    {
        match R::read(&mut self.sock, &mut self.buf, MAX_HEADER_SIZE) {
            R::Full => {
                debug!("Headers too long");
                return false;
            }
            R::Close => {
                return false; // Connection closed
            }
            R::More => {
                let mut headers = [httparse::EMPTY_HEADER; 64];
                let (mut rreq, len) = {
                    let mut req = httparse::Request::new(&mut headers);
                    let len = match req.parse(&self.buf) {
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
                match rreq.headers.get::<ContentLength>() {
                    Some(&ContentLength(x)) => {
                        let clen = x as usize;
                        if self.buf.len() >= len + clen {
                            rreq.body = self.buf[len..len+clen].to_vec();
                            self.buf = self.buf[len+clen..].to_vec();
                        } else {
                            let buf = self.buf[len..len+clen].to_vec();
                            self.buf.truncate(0);
                            self.mode = Mode::ReadContent(rreq, buf, clen);
                            return true;
                        }
                    }
                    None => {
                        self.buf = self.buf[len..].to_vec();
                    }
                }
                if self.buf.len() > 0 {
                    error!("Request pipelining is not implemented");
                    return false;
                }
                match self.resolve(&rreq, context) {
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
                        error!("Can't reregister http socket: {}", e);
                        return false;
                    }
                }
                return true;
            }
            R::Wait => {
                return true;
            }
            R::Error(e) => {
                error!("Error reading request {}", e);
                return false;
            }
        }
    }
}

impl<'a> mio::Handler for Handler<'a> {
    type Timeout = ();
    type Message = ();

    fn readable(&mut self, eloop: &mut EventLoop<Handler>,
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
                    buf: Vec::new(),
                    mode: Mode::ReadHeaders,
                }).ok_or(()).err().expect("Duplicate token in http_inputs");
            }
            tok => {
                let keep = match self.http_inputs.get_mut(&tok) {
                    Some(cli) => {
                        if !cli.read(eloop, &mut self.context) {
                            eloop.deregister(&cli.sock).unwrap();
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

    fn writable(&mut self, eloop: &mut EventLoop<Handler>, tok: Token)
    {
        match tok {
            INPUT => unreachable!(),
            tok => {
                let keep = match self.http_inputs.get_mut(&tok) {
                    Some(cli) => {
                        if !cli.write(eloop) {
                            eloop.deregister(&cli.sock).unwrap();
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
    let mut ctx = Handler {
        input: server,
        http_inputs: Default::default(),
        context: Context {
            stats: stats,
            gossip_cmd: gossip_cmd,
        },
    };
    eloop.run(&mut ctx)
        .map_err(|e| format!("Error running http loop: {}", e))
}
