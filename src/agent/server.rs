use std::str::from_utf8;
use std::net::{SocketAddr, SocketAddrV4};
use std::sync::RwLock;
use std::sync::atomic::{AtomicUsize, Ordering, ATOMIC_USIZE_INIT};
use std::default::Default;
use std::collections::HashMap;

use rustc_serialize::json;
use rustc_serialize::json::ToJson;
use mio;
use hyper::buffer::BufReader;
use mio::buf::{RingBuf, MutBuf};
use mio::TryRead;
use mio::{EventLoop, Token, ReadHint, Handler};
use hyper::http::h1::parse_request;

use super::aio;
use super::scan;
use super::staticfiles;
use super::aio::http;
use super::stats::{Stats};
use super::storage::{StorageStats};
use super::rules::{Query, query};
use super::p2p::Command;


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

#[derive(Debug)]
struct HttpClient {
    token: Token,
    sock: mio::tcp::TcpStream,
    buf: RingBuf,
}

struct Context<'a> {
    input: mio::tcp::TcpListener,
    stats: &'a RwLock<Stats>,
    http_inputs: HashMap<Token, HttpClient>,
}
impl HttpClient {
    fn read(&mut self, eloop: &mut EventLoop<Context>) -> bool {
        debug!("Http {:?}", self);
        if(self.buf.remaining() == 0) {
            debug!("Headers too long");
            return false;
        }
        let res = self.sock.try_read_buf(&mut self.buf);
        match res {
            Ok(Some(x)) => {
                let req = parse_request(&mut BufReader::new(self.buf));
                debug!("GOT REQUEST {:?}", req);
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
                if let Err(e) = eloop.register(&sock, tok) {
                    error!("Error registering accepted connection: {}", e);
                }
                self.http_inputs.insert(tok, HttpClient {
                    token: tok,
                    sock: sock,
                    buf: RingBuf::new(MAX_HEADER_SIZE),
                }).ok_or(()).err().expect("Duplicate token in http_inputs");
            }
            tok => {
                let keep = match self.http_inputs.get_mut(&tok) {
                    Some(cli) => {
                        if !cli.read(eloop) {
                            eloop.deregister(cli.sock);
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
