use std::io::Write;
use std::fmt::Debug;
use std::borrow::{Cow, IntoCow};
use bytes::ByteBuf;

use mime::{Mime};
use hyper::status::StatusCode;
use hyper::version::HttpVersion;
use hyper::header::{ContentType, Headers};
use hyper::uri::RequestUri;
use hyper::method::Method;
use hyper::version::HttpVersion as Version;
use hyper::server::response::Response as HyperResponse;
use hyper::http::h1::HttpWriter::SizedWriter;
use rustc_serialize::Encodable;
use rustc_serialize::json::as_json;


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
    pub fn json<T: Encodable>(body: &T) -> Response
    {
        let mut res = Response::new();
        res.headers.set(
            ContentType(mime!(Application/Json; Charset=Utf8)));
        res.status = StatusCode::Ok;
        res.body = Cow::Owned(format!("{}", as_json(body)).into_bytes());
        return res;
    }
    pub fn to_buf(&mut self, version: HttpVersion) -> Vec<u8> {
        let mut buf = Vec::new();
        {
            let mut res = HyperResponse::new(&mut buf, &mut self.headers);
            res.send(&self.body[..]).unwrap();
        }
        return buf;
    }
}
