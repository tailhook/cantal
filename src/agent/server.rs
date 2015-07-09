use std::str::from_utf8;
use std::io::{Read, Write};
use std::net::{SocketAddr, SocketAddrV4};
use std::sync::RwLock;
use std::collections::HashMap;

use rustc_serialize::json;
use rustc_serialize::json::ToJson;
use mio;
use mio::{Interest, PollOpt};
use mio::{EventLoop, Token, ReadHint};
use mio::util::Slab;

use super::scan;
use super::http;
use super::staticfiles;
use super::websock;
use super::stats::{Stats};
use super::storage::{StorageStats};
use super::rules::{Query, query};
use super::p2p::Command;
use super::http::{NotFound, BadRequest, ServerError, MethodNotAllowed};
use super::http::Request;
use super::util::WriteVec as W;
use super::util::ReadVec as R;


const INPUT: Token = Token(0);
const MAX_CLIENTS: usize = 4096;

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

pub enum Tail {
    Close,
    WebSock,
    Proceed(Vec<u8>),
}

pub enum Client {
    ReadHeaders { buf: Vec<u8> },
    ReadContent { req: Request, body_size: usize },
    Respond { req: Request, tail: Vec<u8> },
    WriteResponse { buf: Vec<u8>, tail: Tail },
    WebSocket { input: Vec<u8>, output: Vec<u8> },
    Close,
}

impl Client {
    pub fn new() -> Client {
        Client::ReadHeaders { buf: Vec::new() }
    }
    pub fn interest(&self) -> Option<Interest> {
        use self::Client::*;
        Some(match self {
            &ReadHeaders { .. } => Interest::readable(),
            &ReadContent { .. } => Interest::readable(),
            &Respond { .. } => Interest::none(),
            &WriteResponse { .. } => Interest::writable(),
            &WebSocket { ref output, .. } if output.len() == 0
            => Interest::readable(),
            &WebSocket { .. } => Interest::readable()|Interest::writable(),
            &Close => return None,
        })
    }
    pub fn do_read(self, sock: &mut mio::tcp::TcpStream, context: &mut Context)
        -> Client
    {
        let mut item = self;
        loop {
            item = item._read(sock, context);
            if item.interest() != Some(Interest::none()) {
                return item;
            }
        }
    }

    pub fn _read(self, sock: &mut mio::tcp::TcpStream, context: &mut Context)
        -> Client
    {
        use self::Client::*;
        match self {
            ReadHeaders { buf }
                => http::read_headers(buf, sock),
            ReadContent { req, body_size }
                => http::read_content(req, body_size, sock),
            Respond { req, tail } => {
                let mut resp = match resolve(&req, context) {
                    Ok(resp) => resp,
                    Err(err) => err.to_response(),
                };
                WriteResponse {
                    buf: resp.to_buf(req.version),
                    tail: if resp.is_websock() { Tail::WebSock }
                          else { Tail::Proceed(tail) },
                }
            }
            WriteResponse { .. } => unreachable!(),
            WebSocket { mut input, output } => {
                match R::read(sock, &mut input, 65536) {
                    R::Full|R::Close => Close,
                    // TODO(tailhook) parse packets
                    R::More|R::Wait
                    => WebSocket { input: input, output: output },
                    R::Error(e) => {
                        error!("Error reading from websocket: {}", e);
                        Close
                    }
                }
            }
            Close => Close,
        }
    }
    pub fn do_write(self, sock: &mut mio::tcp::TcpStream, context: &mut Context)
        -> Client
    {
        use self::Client::*;
        use self::Tail as T;
        match self {
            ReadHeaders { .. } => unreachable!(),
            ReadContent { .. } => unreachable!(),
            Respond { .. } => unreachable!(),
            WriteResponse { buf, tail } => {
                match (W::write(sock, buf), tail) {
                    (W::Done, T::Close) => Close,
                    (W::Done, T::WebSock) => WebSocket { input: vec!(),
                                                         output: vec!() },
                    (W::Done, T::Proceed(chunk)) => {
                        http::read_headers(chunk, sock)
                            .do_read(sock, context)
                    }
                    (W::More(buf), tail) => WriteResponse { buf: buf,
                                                            tail: tail },
                    (W::Close, _) => Close,
                    (W::Error(e), _) => {
                        error!("Error writing response: {}", e);
                        Close
                    }
                }
            },
            WebSocket { input, output } => {
                match W::write(sock, output) {
                    W::Done => WebSocket { input: input, output: Vec::new() },
                    W::More(buf) => WebSocket { input: input, output: buf },
                    W::Close => Close,
                    W::Error(e) => {
                        error!("Error writing response: {}", e);
                        Close
                    }
                }
            }
            Close => Close,
        }
    }
}

struct Handler<'a> {
    input: mio::tcp::TcpListener,
    clients: Slab<(mio::tcp::TcpStream, Option<Client>)>,
    stats: &'a RwLock<Stats>,
    gossip_cmd: mio::Sender<Command>,
}

pub struct Context<'a, 'b: 'a> {
    stats: &'a RwLock<Stats>,
    gossip_cmd: &'a mut mio::Sender<Command>,
    eloop: &'a mut EventLoop<Handler<'b>>,
}

fn resolve(req: &Request, context: &mut Context)
    -> Result<http::Response, Box<http::Error>>
{
    use hyper::method::Method::*;
    use hyper::uri::RequestUri::AbsolutePath as P;
    debug!("Got request {:?} {:?}", req.method, req.uri);
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
        => websock::respond_websock(req, context),
        (&Get, &P(ref x)) if &x[..] == "/status.json"
        => serve_status(req, context),
        (&Get, &P(ref x)) if &x[..] == "/all_processes.json"
        => serve_processes(req, context),
        (&Get, &P(ref x)) if &x[..] == "/all_metrics.json"
        => serve_metrics(req, context),
        (&Get, &P(ref x)) if &x[..] == "/all_peers.json"
        => serve_peers(req, context),
        (&Post, &P(ref x)) if &x[..] == "/query.json"
        => serve_query(req, context),
        (&Post, &P(ref x)) if &x[..] == "/add_host.json"
        => do_add_host(req, context),
        (&Get, _) => Err(Box::new(NotFound) as Box<http::Error>),
        _ => Err(Box::new(MethodNotAllowed) as Box<http::Error>),
    }
}

fn serve_status(_req: &Request, context: &mut Context)
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

fn serve_processes(_req: &Request, context: &mut Context)
    -> Result<http::Response, Box<http::Error>>
{
    let stats = context.stats.read().unwrap();
    Ok(http::Response::json(&ProcessesData {
            boot_time: stats.boot_time,
            all: &stats.processes,
        }))
}

fn serve_metrics(_req: &Request, context: &mut Context)
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

fn serve_peers(_req: &Request, context: &mut Context)
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

fn serve_query(req: &Request, context: &mut Context)
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

fn do_add_host(req: &Request, context: &mut Context)
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

impl<'a> mio::Handler for Handler<'a> {
    type Timeout = ();
    type Message = ();

    fn readable(&mut self, eloop: &mut EventLoop<Handler>,
                tok: Token, _hint: ReadHint)
    {
        let mut context = Context {
            stats: self.stats,
            gossip_cmd: &mut self.gossip_cmd,
            eloop: eloop,
        };
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
                let cli = Client::new();
                debug!("Accepted {:?}", sock.peer_addr());
                let cli_intr = cli.interest().unwrap();
                let ntok = self.clients.insert((sock, Some(cli)))
                    .map_err(|_| error!("Too many clients"));
                if let Ok(cli_token) = ntok {
                    if let Err(e) = context.eloop.register_opt(
                        &self.clients.get(cli_token).unwrap().0,
                        cli_token, cli_intr, PollOpt::level())
                    {
                        error!("Error registering accepted connection: {}", e);
                        self.clients.remove(cli_token);
                    }
                }
            }
            tok => {
                trace!("Readable {:?}", tok);
                if let Some(&mut (ref mut sock, ref mut cliref)) =
                        self.clients.get_mut(tok)
                {
                    let mut cli = cliref.take().unwrap();
                    let old_int = cli.interest();
                    cli = cli.do_read(sock, &mut context);
                    let new_int = cli.interest();
                    *cliref = Some(cli);
                    if old_int == new_int {
                        return
                    }
                    match new_int {
                        Some(x) => {
                            match context.eloop.reregister(sock, tok, x,
                                PollOpt::level())
                            {
                                Ok(_) => return,
                                Err(e) => {
                                    error!("Error on reregister: {}; \
                                            closing connection", e);
                                    context.eloop.deregister(sock).ok();
                                }
                            }
                        }
                        None => {
                            context.eloop.deregister(sock)
                            .map_err(|e| error!("Error on deregister: {}", e))
                            .ok();
                        }
                    }
                } else {
                    error!("Unknown token {:?}", tok);
                    return
                }
                // If we have not returned yet: remove socket
                self.clients.remove(tok);
            }
        }
    }
    fn writable(&mut self, eloop: &mut EventLoop<Handler>, tok: Token)
    {
        let mut context = Context {
            stats: self.stats,
            gossip_cmd: &mut self.gossip_cmd,
            eloop: eloop,
        };
        match tok {
            INPUT => unreachable!(),
            tok => {
                if let Some(&mut (ref mut sock, ref mut cliref)) =
                        self.clients.get_mut(tok)
                {
                    let mut cli = cliref.take().unwrap();
                    let old_int = cli.interest();
                    cli = cli.do_write(sock, &mut context);
                    let new_int = cli.interest();
                    *cliref = Some(cli);
                    if old_int == new_int {
                        return
                    }
                    match new_int {
                        Some(x) => {
                            match context.eloop.reregister(sock, tok, x,
                                PollOpt::level())
                            {
                                Ok(_) => return,
                                Err(e) => {
                                    error!("Error on reregister: {}; \
                                            closing connection", e);
                                    context.eloop.deregister(sock).ok();
                                }
                            }
                        }
                        None => {
                            context.eloop.deregister(sock)
                            .map_err(|e| error!("Error on deregister: {}", e))
                            .ok();
                        }
                    }
                } else {
                    error!("Unknown token {:?}", tok);
                    return
                }
                // If we have not returned yet: remove socket
                self.clients.remove(tok);
            }
        }
    }
}

pub fn run_server(stats: &RwLock<Stats>, host: &str, port: u16,
    gossip_cmd: mio::Sender<Command>)
    -> Result<(), String>
{
    let server = mio::tcp::TcpListener::bind(&SocketAddr::V4(
        SocketAddrV4::new(host.parse().unwrap(), port))).unwrap();
    let mut eloop = EventLoop::new().unwrap();
    eloop.register(&server, INPUT).unwrap();
    let mut ctx = Handler {
        input: server,
        clients: Slab::new_starting_at(Token(1), MAX_CLIENTS),
        stats: stats,
        gossip_cmd: gossip_cmd,
    };
    eloop.run(&mut ctx)
        .map_err(|e| format!("Error running http loop: {}", e))
}
