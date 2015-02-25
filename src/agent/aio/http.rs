//  This implementation is an implementation of subset of HTTP.
//  It *may be unsafe* to expose this implementation to the untrusted internet

use std::vec::CowVec;
use std::str::from_utf8;
use std::fmt::Display;
use std::borrow::{Cow, IntoCow};
use std::os::unix::Fd;
use serialize::json::as_pretty_json;
use serialize::Encodable;

use super::{HttpHandler, HandlerResult};
use super::lowlevel::{read_to_vec, ReadResult};


const MAX_HEADERS_SIZE: usize = 16384;


//  We don't do generic HTTP, so we only need these specific methods
#[derive(Show, Copy, PartialEq, Eq)]
pub enum Method {
    Get,
    Unknown,
}


#[derive(Show, Copy, PartialEq, Eq)]
pub enum Version {
    Http10,
    Http11,
}

pub enum TransferEncoding {
    Identity,
    Chunked,
}

#[derive(Show)]
pub enum Error {
    BadRequest(&'static str),
    NotFound,
    MethodNotAllowed,
    ServerError(&'static str),
}

#[derive(Show, Copy)]
pub enum Status {
    Ok, // 200
    BadRequest, // 400
    NotFound, // 404
    MethodNotAllowed, // 405
    ServerError, // 500
}

enum ResponseBody<'a> {
    Empty,
    Chunk(CowVec<'a, u8>),
    Text(&'a Display),
}


#[derive(Show)]
pub struct RequestLine<'a>(Method, &'a str, Version);

pub struct Stream<'a> {
    pub fd: Fd,
    handler: HttpHandler<'a>,
    buf: Vec<u8>,
}

#[derive(Show)]
pub struct Request<'a> {
    request_line: RequestLine<'a>,
    //headers: Vec<(&'a str, &'a str)>,
    //transfer_encoding: TransferEncoding,
    content_length: usize,
    close: bool,
}

#[derive(Show)]
pub struct Response {
    buf: Vec<u8>,
}

pub struct ResponseBuilder<'a> {
    version: Version,
    status: Status,
    body: ResponseBody<'a>,
}

pub struct RequestParser<'a> {
    req: Request<'a>,
    has_content_length: bool,
}

impl<'a> RequestLine<'a> {
    fn version(&self) -> Version {
        let RequestLine(_, _, ver) = *self;
        return ver;
    }
}

impl<'a> Request<'a> {
    pub fn uri<'x>(&'x self) -> &'x str {
        let RequestLine(_, uri, _) = self.request_line;
        return uri;
    }
}

impl<'a> RequestParser<'a> {
    fn start<'x>(req_line: RequestLine<'x>) -> RequestParser<'x>
    {
        return RequestParser {
            req: Request {
                close: req_line.version() == Version::Http10,
                request_line: req_line,
                content_length: 0,
            },
            has_content_length: false,
        };
    }
    fn add_header(&mut self, name: &str, value: &str) -> Result<(), Error>
    {
        Ok(())
    }
    fn take(self) -> Request<'a> {
        return self.req;
    }
}

impl<'a> Stream<'a> {
    pub fn new<'x>(fd: Fd, handler: HttpHandler<'x>) -> Stream<'x> {
        return Stream {
            fd: fd,
            handler: handler,
            buf: Vec::new(),
        }
    }
    pub fn read_http(&mut self) -> HandlerResult {
        match read_to_vec(self.fd, &mut self.buf) {
            ReadResult::Read(start, end) => {
                let check_start = if start > 3 { start - 3 } else { 0 };
                if end - check_start < 4 {
                    if self.buf.len() > MAX_HEADERS_SIZE {
                    } else {
                        return HandlerResult::ContinueRead;
                    }
                }
                for i in range(check_start, end - 3) {
                    if self.buf.slice(i, i+4) == b"\r\n\r\n" {
                        match self.parse_request(self.buf.slice(0, i+2)) {
                            Ok(req) => {
                                match (*self.handler)(&req) {
                                    Ok(resp) => {
                                        return HandlerResult::SendAndClose(
                                            resp.buf);
                                    }
                                    Err(e) => {
                                        let mut builder = ResponseBuilder::new(
                                            &req, e.status());
                                        builder.set_body(e.body().as_bytes());
                                        return HandlerResult::SendAndClose(
                                            builder.take().buf);
                                    }
                                };
                            }
                            Err(e) => {
                                // TODO(tailhook) implement HTTP errors
                                info!("Error parsing request: {:?}", e);
                                return HandlerResult::Close;
                            }
                        }
                    }
                }
                HandlerResult::ContinueRead
            }
            ReadResult::Fatal(err) => {
                error!("Error handling connection (fd: {}): {:?}",
                    self.fd, err);
                HandlerResult::Close
            }
            ReadResult::NoData => HandlerResult::ContinueRead,
            ReadResult::Closed => HandlerResult::Close,
        }
    }

    pub fn _request_line<'x>(&self, chunk: &'x str)
        -> Result<RequestLine<'x>, Error>
    {
        let mut pieces = chunk.trim().words();
        let meth = match pieces.next().unwrap() {
            "GET" => Method::Get,
            _ => return Err(Error::MethodNotAllowed),
        };
        let uri = try!(pieces.next()
            .ok_or(Error::BadRequest("No URI specified")));
        let ver = match pieces.next() {
            Some("HTTP/1.0") => Version::Http10,
            Some("HTTP/1.1") => Version::Http11,
            _ => return Err(Error::BadRequest("Bad HTTP version")),
        };
        Ok(RequestLine(meth, uri, ver))
    }

    pub fn parse_request<'x>(&self, headers: &'x [u8])
        -> Result<Request<'x>, Error>
    {
        let line_end = headers.position_elem(&b'\r').unwrap();
        let req_line = try!(from_utf8(&headers[..line_end])
                 .map_err(|_| Error::BadRequest("Can't decode headers")));
        let mut req_parser = RequestParser::start(
            try!(self._request_line(req_line)));
        if headers[line_end+1] != b'\n' {
            return Err(Error::BadRequest("Wrong end of line"));
        }
        let mut pos = line_end + 2;
        if pos < headers.len() &&
            (headers[pos] == b' ' || headers[pos] == b'\t')
        {
            return Err(Error::BadRequest(
                "Continuation line without headers"));
        }
        while pos < headers.len() {
            let start = pos;
            while headers[pos] != b':' {
                if headers[pos] == b'\r' || headers[pos] == b'\n' {
                    return Err(Error::BadRequest(
                        "Header line without colon"));
                }
                pos += 1;
            }
            let name = try!(from_utf8(&headers[start..pos])
                 .map_err(|_| Error::BadRequest("Can't decode headers")));
            let vstart = pos + 1;
            while headers[pos] != b'\r' {
                pos += 1;
                // We know that there is always '\r\n' at the end
                if headers[pos] == b'\r' && headers[pos+1] == b'\n' &&
                    headers.len() > pos+2 &&
                    (headers[pos+2] == b' ' || headers[pos+2] == b'\t') {
                    pos += 1; // continuation line
                }
            }
            let value = try!(from_utf8(&headers[vstart..pos])
                 .map_err(|_| Error::BadRequest("Can't decode headers")));
            // We know that there is always '\r\n' at the end
            if headers[pos+1] != b'\n' {
                return Err(Error::BadRequest("Wrong end of line"));
            }
            pos += 2;
            try!(req_parser.add_header(name, value));
        }
        return Ok(req_parser.take());
    }
}

impl Error {
    fn status(&self) -> Status {
        match *self {
            Error::BadRequest(_) => Status::BadRequest,
            Error::NotFound => Status::NotFound,
            Error::MethodNotAllowed => Status::MethodNotAllowed,
            Error::ServerError(_) => Status::ServerError,
        }
    }
    fn body(&self) -> &'static str {
        match *self {
            Error::BadRequest(val) => val,
            Error::NotFound => "Page Not Found",
            Error::MethodNotAllowed => "Method Not Allowed",
            Error::ServerError(val) => val,
        }
    }
}

impl Status {
    fn status_code(self) -> u32 {
        match self {
            Status::Ok => 200,
            Status::NotFound => 404,
            Status::BadRequest => 400,
            Status::MethodNotAllowed => 405,
            Status::ServerError => 500,
        }
    }
    fn status_text(self) -> &'static str {
        match self {
            Status::Ok => "OK",
            Status::NotFound => "Not Found",
            Status::BadRequest => "Bad Request",
            Status::MethodNotAllowed => "Method Not Allowed",
            Status::ServerError => "Internal Server Error",
        }
    }
}

impl Version {
    fn text(self) -> &'static str {
        match self {
            Version::Http10 => "HTTP/1.0",
            Version::Http11 => "HTTP/1.1",
        }
    }
}

impl<'a> ResponseBuilder<'a> {
    pub fn new<'x>(req: &Request, status: Status) -> ResponseBuilder<'x> {
        return ResponseBuilder {
            version: req.request_line.version(),
            status: status,
            body: ResponseBody::Empty,
        };
    }
    pub fn set_body<T:IntoCow<'a, [u8]>>(&mut self, x: T)
    {
        self.body = ResponseBody::Chunk(x.into_cow());
    }
    pub fn set_body_text(&mut self, x: &'a Display)
    {
        self.body = ResponseBody::Text(x);
    }
    pub fn take(self) -> Response {
        let buf = match self.body {
            ResponseBody::Empty => format!("{} {} {}\r\n\
                        Content-Length: 0\r\n\
                        Connection: close\r\n\
                        \r\n",
                        self.version.text(),
                        self.status.status_code(),
                        self.status.status_text()).into_bytes(),
            ResponseBody::Chunk(body) => format!("{} {} {}\r\n\
                        Content-Length: {}\r\n\
                        Connection: close\r\n\
                        \r\n",
                        self.version.text(),
                        self.status.status_code(),
                        self.status.status_text(),
                        body.len()
                    ).into_bytes() + (*body).as_slice(),
            ResponseBody::Text(body) => format!("{} {} {}\r\n\
                        Connection: close\r\n\
                        \r\n{}",  // note, no Content-Length, use Conn: close
                        self.version.text(),
                        self.status.status_code(),
                        self.status.status_text(),
                        body).into_bytes(),
        };
        return Response {
            buf: buf,
        };
    }
}

pub fn reply_json<T:Encodable>(req: &Request, val: &T) -> Response {
    ResponseBuilder {
        version: req.request_line.version(),
        status: Status::Ok,
        body: ResponseBody::Text(&as_pretty_json(val)),
    }.take()
}
