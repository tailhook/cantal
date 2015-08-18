use std::io;
use std::sync::{Arc, RwLock};
use std::mem::replace;
use std::str::from_utf8;
use std::io::{Read, Write};
use std::net::{SocketAddr,Ipv4Addr, SocketAddrV4};
use std::collections::{HashMap, HashSet};

use rustc_serialize::json;
use rustc_serialize::json::ToJson;
use probor;
use mio;
use mio::Sender;
use mio::{EventSet, PollOpt};
use mio::{EventLoop, Token};
use mio::tcp::TcpStream;
use mio::util::Slab;

use query::{Filter, Rule, query_history};
use super::p2p;
use super::http;
use super::staticfiles;
use super::respond;
use super::remote;
use super::websock;
use super::ioutil::Poll;
use super::stats::Stats;
use super::error::Error;
use super::http::{NotFound, BadRequest, ServerError, MethodNotAllowed};
use super::http::Request;
use super::util::WriteVec as W;
use super::util::ReadVec as R;
use super::deps::{Dependencies, LockedDeps};
use super::websock::{InputMessage, OutputMessage};


const INPUT: Token = Token(0);
const MAX_HTTP_CLIENTS: usize = 4096;
const MAX_WEBSOCK_CLIENTS: usize = 4096;
const MAX_WEBSOCK_MESSAGE: usize = 16384;
pub const MAX_OUTPUT_BUFFER: usize = 1_073_741_824;


#[derive(Debug)]
pub enum Tail {
    Close,
    WebSock,
    Proceed(Vec<u8>),
}

#[derive(Debug)]
pub enum Client {
    ReadHeaders { buf: Vec<u8> },
    ReadContent { req: Request, body_size: usize },
    Respond { req: Request, tail: Vec<u8> },
    WriteResponse { buf: Vec<u8>, tail: Tail },
    WebSockStart,
    Close,
}

struct WebSocket {
    sock: TcpStream,
    input: Vec<u8>,
    output: Vec<u8>,
    subscriptions: HashSet<Filter>,
}

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum Interest {
    Readable,
    Writable,
    Continue,
    SwitchWebsock,
    Close,
}

impl Interest {
    fn event_set(self) -> EventSet {
        match self {
            Interest::Readable => EventSet::readable(),
            Interest::Writable => EventSet::writable(),
            _ => unreachable!(),
        }
    }
}

impl Client {
    pub fn new() -> Client {
        Client::ReadHeaders { buf: Vec::new() }
    }
    pub fn interest(&self) -> Interest {
        use self::Client::*;
        use self::Interest as I;
        match self {
            &ReadHeaders { .. } => I::Readable,
            &ReadContent { .. } => I::Readable,
            &Respond { .. } => I::Continue,
            &WriteResponse { .. } => I::Writable,
            &WebSockStart => I::SwitchWebsock,
            &Close => I::Close,
        }
    }
    pub fn do_read(self, sock: &mut mio::tcp::TcpStream, context: &mut Context)
        -> Client
    {
        let mut item = self._read(sock, context);
        while item.interest() == Interest::Continue {
            item = item._read(sock, context);
        }
        return item;
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
            // TODO(tailhook) probably move respond out of here
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
            WebSockStart => WebSockStart,
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
                    (W::Done, T::WebSock) => WebSockStart,
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
            WebSockStart => WebSockStart,
            Close => Close,
        }
    }
}

pub struct Handler {
    input: mio::tcp::TcpListener,
    clients: Slab<(mio::tcp::TcpStream, Option<Client>)>,
    websockets: Slab<WebSocket>,
    deps: Dependencies,
}

pub struct Context<'a, 'b: 'a> {
    pub deps: &'b mut Dependencies,
    pub eloop: &'a mut EventLoop<Handler>,
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
        => respond::serve_status(req, context),
        (&Get, &P(ref x)) if &x[..] == "/all_processes.json"
        => respond::serve_processes(req, context),
        (&Get, &P(ref x)) if &x[..] == "/all_metrics.json"
        => respond::serve_metrics(req, context),
        (&Get, &P(ref x)) if &x[..] == "/all_peers.json"
        => respond::serve_peers(req, context),
        (&Get, &P(ref x)) if &x[..] == "/peers_with_remote.json"
        => respond::serve_peers_with_remote(req, context),
        (&Get, &P(ref x)) if &x[..] == "/remote_stats.json"
        => respond::serve_remote_stats(req, context),
        (&Post, &P(ref x)) if &x[..] == "/query.cbor"
        => respond::serve_query(req, context),
        (&Post, &P(ref x)) if &x[..] == "/remote/query_by_host.json"
        => remote::respond::serve_query_by_host(req, context),
        (&Post, &P(ref x)) if &x[..] == "/add_host.json"
        => do_add_host(req, context),
        // TODO(tailhook) this should be post
        (&Post, &P(ref x)) if &x[..] == "/start_remote.json"
        => do_start_remote(req, context),
        (&Get, _) => Err(Box::new(NotFound) as Box<http::Error>),
        _ => Err(Box::new(MethodNotAllowed) as Box<http::Error>),
    }
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
   .and_then(|x| context.deps.get::<Sender<_>>()
                        .unwrap().send(p2p::Command::AddGossipHost(x))
   .map_err(|e| error!("Error sending to p2p loop: {:?}", e))
   .map_err(|_| ServerError::err("Notify Error")))
   .and_then(|_| {
        Ok(http::Response::json(&vec![
            (String::from("ok"), true)
        ].into_iter().collect::<HashMap<_, _>>().to_json()))
    })
}

fn do_start_remote(_req: &Request, context: &mut Context)
    -> Result<http::Response, Box<http::Error>>
{
    remote::ensure_started(context);
    Ok(http::Response::json(&vec![
        (String::from("ok"), true)
    ].into_iter().collect::<HashMap<_, _>>().to_json()))
}

#[derive(Debug)]
pub enum Timer {
    ReconnectPeer(Token),
    ResetPeer(Token),
    RemoteCollectGarbage,
}

#[derive(Debug)]
pub enum Message {
    ScanComplete,
    NewHost(Ipv4Addr, u16),
}

impl Handler {
    fn try_http(&mut self, tok: Token, ev: EventSet,
        eloop: &mut EventLoop<Handler>)
        -> bool
    {
        use self::Interest::*;
        let mut context = Context {
            deps: &mut self.deps,
            eloop: eloop,
        };
        let new_int = if let Some(&mut (ref mut sock, ref mut cliref)) =
                self.clients.get_mut(tok)
        {
            let mut cli = cliref.take().unwrap();
            let old_int = cli.interest();
            if ev.is_readable() {
                cli = cli.do_read(sock, &mut context);
            }
            if ev.is_writable() {
                cli = cli.do_write(sock, &mut context);
            }
            let new_int = cli.interest();
            *cliref = Some(cli);
            if old_int == new_int {
                return true;
            }
            match new_int {
                Readable|Writable => {
                    match context.eloop.reregister(sock, tok,
                        new_int.event_set(), PollOpt::level())
                    {
                        Ok(_) => return true,
                        Err(e) => {
                            error!("Error on reregister: {}; \
                                    closing connection", e);
                            panic!("Error on reregister: {}; \
                                    closing connection", e);
                        }
                    }
                }
                Continue => unreachable!(),
                SwitchWebsock|Close => {
                    context.eloop.remove(sock);
                }
            }
            new_int
        } else {
            return false;
        };
        // If we have not returned yet: remove socket
        if new_int == Close {
            self.clients.remove(tok);
        } else if new_int == SwitchWebsock {
            let (sock, _) = self.clients.remove(tok).unwrap();

            // Send beacon at start
            let mut buf = Vec::new();
            let beacon = websock::beacon(&context.deps);
            websock::write_binary(&mut buf, &beacon);

            let ntok = self.websockets.insert(WebSocket {
                sock: sock,
                input: Vec::new(),
                output: buf,
                subscriptions: HashSet::new(),
                })
                .map_err(|_| error!("Too many websock clients"));
            if let Ok(cli_token) = ntok {
                context.eloop.add(
                    &self.websockets.get(cli_token).unwrap().sock,
                    cli_token, true, true);
            }
        }
        return true;
    }
    fn try_websock(&mut self, tok: Token, ev: EventSet,
        eloop: &mut EventLoop<Handler>)
        -> bool
    {
        use self::Interest::*;
        let mut context = Context {
            deps: &mut self.deps,
            eloop: eloop,
        };
        if let Some(ref mut wsock) = self.websockets.get_mut(tok) {
            if ev.is_writable() {
                let buf = replace(&mut wsock.output, Vec::new());
                match W::write(&mut wsock.sock, buf) {
                    W::Done => {
                        context.eloop.modify(&wsock.sock, tok, true, false);
                        return true;
                    }
                    W::More(buf) => {
                        wsock.output = buf;
                    }
                    W::Close => {}
                    W::Error(err) => {
                        debug!("Error writing to websock: {}", err);
                    }
                }
            }
            if ev.is_readable() {
                match R::read(&mut wsock.sock, &mut wsock.input,
                              MAX_WEBSOCK_MESSAGE)
                {
                    R::Wait => {
                        return true;
                    }
                    R::More => {
                        // TODO(tailhook) try parse message
                        loop {
                            let msg: Option<websock::InputMessage>;
                            msg = websock::parse_message(&mut wsock.input,
                                &mut context,
                                |opcode, msg, _ctx| {
                                    if opcode == websock::Opcode::Binary {
                                        probor::from_slice(msg)
                                        .map_err(|e| error!(
                                            "Error decoding msg {:?}", e))
                                        .ok()
                                    } else {
                                        None
                                    }
                                });
                            if let Some(msg) = msg {
                                debug!("Websock input {:?}", msg);
                                match msg {
                                    InputMessage::Subscribe(sub, depth) => {
                                        // TODO(tailhook) move it to remote
                                        if depth != 0 { // not sure check is ok
                                            let val = query_history(
                                                &Rule {
                                                    series: sub.clone(),
                                                    extract: remote::EXTRACT,
                                                    functions: Vec::new(),
                                                },
                                                &context.deps.read::<Stats>()
                                                    .history);
                                            let msg = probor::to_buf(
                                                &OutputMessage::Stats(
                                                    vec![val]));
                                            let start = wsock.output.len() == 0;
                                            websock::write_binary(
                                                &mut wsock.output, &msg);
                                            if start {
                                                context.eloop.modify(
                                                    &wsock.sock, tok,
                                                    true, true);
                                            }
                                        }
                                        wsock.subscriptions.insert(sub);
                                    }
                                    InputMessage::Unsubscribe(sub) => {
                                        wsock.subscriptions.remove(&sub);
                                    }
                                }
                            } else {
                                break;
                            }
                        }
                        return true;
                    }
                    R::Full|R::Close => {} // exit from if and close socket
                    R::Error(err) => {
                        debug!("Error reading from websock: {}", err);
                    }
                }
            } else {
                return true;
            }
            context.eloop.remove(&wsock.sock);
        } else {
            return false
        }
        self.websockets.remove(tok);
        return true;
    }
    fn try_remote(&mut self, tok: Token, ev: EventSet,
        eloop: &mut EventLoop<Handler>)
        -> bool
    {
        let mut context = Context {
            deps: &mut self.deps,
            eloop: eloop,
        };
        remote::try_io(tok, ev, &mut context)
    }

    fn send_all(&mut self, eloop: &mut EventLoop<Handler>, msg: &[u8]) {
        for tok in (1+MAX_HTTP_CLIENTS .. 1+MAX_HTTP_CLIENTS+MAX_WEBSOCK_CLIENTS) {
            let tok = Token(tok);
            if let Some(ref mut wsock) = self.websockets.get_mut(tok) {
                if wsock.output.len() < MAX_OUTPUT_BUFFER {
                    let start = wsock.output.len() == 0;
                    websock::write_binary(&mut wsock.output, msg);
                    if start {
                        eloop.modify(&wsock.sock, tok, true, true);
                    }
                    continue;
                }
                debug!("Websocket buffer overflow");
                eloop.remove(&wsock.sock);
            } else {
                continue;
            }
            self.websockets.remove(tok);
        }
    }
}

impl mio::Handler for Handler {
    type Timeout = Timer;
    type Message = Message;

    fn ready(&mut self, eloop: &mut EventLoop<Handler>,
        tok: Token, ev: EventSet)
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
                let cli = Client::new();
                debug!("Accepted {:?}", sock.peer_addr());
                let cli_intr = cli.interest();
                let ntok = self.clients.insert((sock, Some(cli)))
                    .map_err(|_| error!("Too many clients"));
                if let Ok(cli_token) = ntok {
                    if let Err(e) = eloop.register_opt(
                        &self.clients.get(cli_token).unwrap().0,
                        cli_token, cli_intr.event_set(), PollOpt::level())
                    {
                        error!("Error registering accepted connection: {}", e);
                        self.clients.remove(cli_token);
                    }
                }
            }
            tok => {
                if !self.try_http(tok, ev, eloop) &&
                   !self.try_websock(tok, ev, eloop) &&
                   !self.try_remote(tok, ev, eloop) {
                    error!("Wrong token {:?}", tok);
                }
            }
        }
    }

    fn notify(&mut self, eloop: &mut EventLoop<Handler>, msg: Message) {
        match msg {
            Message::ScanComplete => {
                let beacon = websock::beacon(&self.deps);
                self.send_all(eloop, &beacon);

                // TODO(tailhook) refactor me PLEASE!!!
                let stats = self.deps.read::<Stats>();
                for tok in (1+MAX_HTTP_CLIENTS .. 1+MAX_HTTP_CLIENTS+MAX_WEBSOCK_CLIENTS) {
                    let tok = Token(tok);
                    if let Some(ref mut wsock) = self.websockets.get_mut(tok) {
                        if wsock.subscriptions.len() <= 0 {
                            continue;
                        }
                        if wsock.output.len() < MAX_OUTPUT_BUFFER {
                            let start = wsock.output.len() == 0;
                            let mut buf = Vec::new();
                            for sub in wsock.subscriptions.iter() {
                                buf.push(query_history(&Rule {
                                    series: sub.clone(),
                                    extract: remote::EXTRACT_ONE,
                                    functions: Vec::new(),
                                }, &stats.history));
                            }
                            let msg = probor::to_buf(
                                &OutputMessage::Stats(buf));

                            websock::write_binary(&mut wsock.output, &msg);
                            if start {
                                eloop.modify(&wsock.sock, tok, true, true);
                            }
                            continue;
                        }
                        debug!("Websocket buffer overflow");
                        eloop.remove(&wsock.sock);
                    } else {
                        continue;
                    }
                    self.websockets.remove(tok);
                }
            }
            Message::NewHost(ip, port) => {
                {
                    let mut context = Context {
                        deps: &mut self.deps,
                        eloop: eloop,
                    };
                    remote::add_peer(
                        SocketAddr::V4(SocketAddrV4::new(ip, port)),
                        &mut context);
                }
                let new_peer = websock::new_peer(ip, port);
                self.send_all(eloop, &new_peer);
            }
        }
    }

    fn timeout(&mut self, eloop: &mut EventLoop<Handler>, timeout: Timer) {
        let mut context = Context {
            deps: &mut self.deps,
            eloop: eloop,
        };
        match timeout {
            Timer::ReconnectPeer(tok) => {
               remote::reconnect_peer(tok, &mut context);
            }
            Timer::ResetPeer(tok) => {
               remote::reset_peer(tok, &mut context);
            }
            Timer::RemoteCollectGarbage => {
               remote::garbage_collector(&mut context);
            }
        }
    }
}

pub struct Init {
    input: mio::tcp::TcpListener,
    eloop: EventLoop<Handler>,
}

pub fn server_init(deps: &mut Dependencies, host: &str, port: u16)
    -> Result<Init, Error>
{
    let server = try!(mio::tcp::TcpListener::bind(&SocketAddr::V4(
        SocketAddrV4::new(try!(host.parse()), port))));
    let mut eloop = try!(EventLoop::new());
    try!(eloop.register(&server, INPUT));
    deps.insert(eloop.channel());
    deps.insert(Arc::new(RwLock::new(None::<remote::Peers>)));
    Ok(Init {
        input: server,
        eloop: eloop,
    })
}

pub fn server_loop(init: Init, deps: Dependencies)
    -> Result<(), io::Error>
{
    let mut eloop = init.eloop;
    eloop.run(&mut Handler {
        input: init.input,
        clients: Slab::new_starting_at(Token(1),
                                       MAX_HTTP_CLIENTS),
        websockets: Slab::new_starting_at(Token(1+MAX_HTTP_CLIENTS),
                                        MAX_WEBSOCK_CLIENTS),
        deps: deps,
    })
}
