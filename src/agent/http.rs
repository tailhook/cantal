use std::fmt::Debug;
use std::borrow::{Cow};

use mio::tcp::TcpStream;
use mime::{Mime};
use httparse;
use unicase::UniCase;
use hyper::error::Error as HyperError;
use hyper::status::StatusCode;
use hyper::version::HttpVersion;
use hyper::header::{ContentType, Headers, Connection, ContentLength};
use hyper::header::{Upgrade, Protocol, ProtocolName};
use hyper::header::ConnectionOption::ConnectionHeader;
use hyper::uri::RequestUri;
use hyper::method::Method;
use hyper::version::HttpVersion as Version;
use hyper::server::response::Response as HyperResponse;
use websocket::header::{WebSocketAccept, WebSocketKey};
use rustc_serialize::Encodable;
use rustc_serialize::json::as_json;

use super::util::ReadVec as R;
use super::util::Consume;
use super::server::{Client, Tail};


const MAX_HEADER_SIZE: usize = 16384;


pub trait Error: Debug {
    fn to_response(&self) -> Response;
}

#[derive(Debug)]
pub struct BadRequest(pub &'static str);
#[derive(Debug)]
pub struct NotFound;
#[derive(Debug)]
pub struct MethodNotAllowed;
#[derive(Debug)]
pub struct ServerError(pub &'static str);

impl Error for BadRequest {
    fn to_response(&self) -> Response {
        return Response::static_mime_str(StatusCode::BadRequest,
            mime!(Text/Plain; Charset=Utf8),
            self.0);
    }
}
impl BadRequest {
    pub fn err(s: &'static str) -> Box<Error> {
        Box::new(BadRequest(s)) as Box<Error>
    }
}
impl Error for NotFound {
    fn to_response(&self) -> Response {
        return Response::static_mime_str(StatusCode::NotFound,
            mime!(Text/Plain; Charset=Utf8),
            "Page Not Found");
    }
}
impl ServerError {
    pub fn err(s: &'static str) -> Box<Error> {
        Box::new(ServerError(s)) as Box<Error>
    }
}
impl Error for ServerError {
    fn to_response(&self) -> Response {
        return Response::static_mime_str(StatusCode::InternalServerError,
            mime!(Text/Plain; Charset=Utf8),
            self.0);
    }
}

impl Error for MethodNotAllowed {
    fn to_response(&self) -> Response {
        return Response::static_mime_str(StatusCode::MethodNotAllowed,
            mime!(Text/Plain; Charset=Utf8),
            "Method Not Allowed");
    }
}


trait Responding {
    fn respond(self) -> Client;
}

impl Responding for Box<Error> {
    fn respond(self) -> Client {
        Client::WriteResponse {
            buf: self.to_response().to_buf(Version::Http10),
            tail: Tail::Close,
        }
    }
}

#[derive(Debug)]
pub struct Response {
    status: StatusCode,
    headers: Headers,
    body: Cow<'static, [u8]>,
}

pub struct Request {
    pub method: Method,
    pub uri: RequestUri,
    pub version: Version,
    pub headers: Headers,
    pub body: Vec<u8>,
}

impl Response {
    fn new() -> Response {
        Response {
            status: StatusCode::NotImplemented,
            headers: Headers::new(),
            body: Cow::Borrowed(b""),
        }
    }
    pub fn static_mime_str(status: StatusCode, mime: Mime, body: &'static str)
        -> Response
    {
        Response::static_mime(status, mime, Cow::Borrowed(body.as_bytes()))
    }
    pub fn static_mime_vec(status: StatusCode, mime: Mime, body: Vec<u8>)
        -> Response
    {
        Response::static_mime(status, mime, Cow::Owned(body))
    }
    pub fn static_mime(status: StatusCode, mime: Mime,
        body: Cow<'static, [u8]>)
        -> Response
    {
        let mut res = Response::new();
        res.headers.set(ContentType(mime));
        res.status = status;
        res.body = body;
        return res;
    }
    pub fn accept_websock(key: &WebSocketKey) -> Response {
        let mut resp = Response::new();
        resp.status = StatusCode::SwitchingProtocols;
        resp.headers.set(Upgrade(vec![
            Protocol::new(ProtocolName::WebSocket, None),
            ]));
        resp.headers.set(Connection(vec![
            ConnectionHeader(UniCase("Upgrade".to_string()))]));
        resp.headers.set(WebSocketAccept::new(key));
        return resp;
    }
    pub fn json<T: Encodable>(body: &T) -> Response
    {
        let mut res = Response::new();
        res.headers.set(
            ContentType(mime!(Application/Json; Charset=Utf8)));
        res.status = StatusCode::Ok;
        res.body = Cow::Owned(format!("{}", as_json(body)).into_bytes());
        return res;
    }
    pub fn is_websock(&self) -> bool {
        self.status == StatusCode::SwitchingProtocols
    }
    pub fn to_buf(&mut self, version: HttpVersion) -> Vec<u8> {
        let mut buf = Vec::new();
        {
            let mut res = HyperResponse::new(&mut buf, &mut self.headers);
            *res.status_mut() = self.status;
            res.version = version;
            res.send(&self.body[..]).unwrap();
        }
        return buf;
    }
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

fn try_parse(buf: &Vec<u8>)
    -> Result<Option<(Request, usize)>, Box<Error>>
{
    let mut headers = [httparse::EMPTY_HEADER; 64];
    let mut req = httparse::Request::new(&mut headers);
    match req.parse(&buf) {
        Ok(httparse::Status::Complete(len)) => {
            match make_request(req) {
                Ok(rreq) => Ok(Some((rreq, len))),
                Err(e) => {
                    debug!("Error decoding request: {:?}", e);
                    // TODO(tailhook) respond with bad request
                    Err(BadRequest::err("Invalid headers"))
                }
            }
        }
        Ok(httparse::Status::Partial) => {
            trace!("Partial read");
            Ok(None)
        }
        Err(err) => {
            debug!("Error while reading request: {:?}", err);
            // TODO(tailhook) respond with bad request
            Err(BadRequest::err("Headers too long"))
        }
    }
}

pub fn read_headers(mut buf: Vec<u8>, sock: &mut TcpStream) -> Client {
    match R::read(sock, &mut buf, MAX_HEADER_SIZE) {
        R::Full => {
            BadRequest::err("Headers too long").respond()
        }
        R::Close => {
            Client::Close
        }
        R::More => {
            match try_parse(&buf) {
                Ok(Some((mut req, header_len))) => {
                    buf.consume(header_len);
                    match req.headers.get::<ContentLength>() {
                        Some(&ContentLength(x)) => {
                            let clen = x as usize;
                            if buf.len() >= clen {
                                // TODO(tailhook) use split_off
                                req.body = buf[..clen].to_vec();
                                buf.consume(clen);
                                Client::Respond { req: req, tail: buf }
                            } else {
                                req.body = buf;
                                Client::ReadContent {
                                    req: req,
                                    body_size: clen,
                                }
                            }
                        }
                        None => {
                            Client::Respond { req: req, tail: buf }
                        }
                    }
                }
                Ok(None) => Client::ReadHeaders { buf: buf },
                Err(e) => e.respond()
            }
        }
        R::Wait => {
            Client::ReadHeaders { buf: buf }
        }
        R::Error(e) => {
            error!("Error reading request {}", e);
            Client::Close
        }
    }
}

pub fn read_content(mut req: Request, body_size: usize,
    sock: &mut TcpStream)
    -> Client
{
    match R::read(sock, &mut req.body, body_size) {
        R::Full => {
            Client::Respond { req: req, tail: Vec::new() }
        }
        R::More|R::Wait => {
            Client::ReadContent { req: req, body_size: body_size }
        }
        R::Close => {
            debug!("Connection closed while reading request body");
            Client::Close
        }
        R::Error(e) => {
            debug!("Error while reading request body: {:?}", e);
            Client::Close
        }
    }
}
