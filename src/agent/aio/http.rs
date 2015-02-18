//  This implementation is an implementation of subset of HTTP.
//  It *may be unsafe* to expose this implementation to the untrusted internet

use std::vec::CowVec;
use std::str::from_utf8;
use std::borrow::{Cow, IntoCow};
use std::os::unix::Fd;
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
    MethodNotAllowed,
}

#[derive(Show, Copy)]
pub enum Status {
    Ok,
    NotFound,
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
    body: CowVec<'a, u8>,
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
                                        unimplemented!();
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

impl Status {
    fn status_code(self) -> u32 {
        match self {
            Status::Ok => 200,
            Status::NotFound => 404,
        }
    }
    fn status_text(self) -> &'static str {
        match self {
            Status::Ok => "OK",
            Status::NotFound => "Not Found",
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
            body: b"".into_cow(),
        };
    }
    pub fn set_body<T:IntoCow<'a, Vec<u8>, [u8]>>(&mut self, x: T)
    {
        self.body = x.into_cow();
    }
    pub fn take(self) -> Response {
        return Response {
            buf: format!("{} {} {}\r\n\
                    Content-Length: {}\r\n\
                    Connection: close\r\n\
                    \r\n",
                    self.version.text(),
                    self.status.status_code(),
                    self.status.status_text(),
                    self.body.len()
                ).into_bytes() + self.body.as_slice(),
        };
    }
}
