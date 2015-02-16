//  This implementation is an implementation of subset of HTTP.
//  It *may be unsafe* to expose this implementation to the untrusted internet

use std::str::from_utf8;
use std::os::unix::Fd;
use super::{HttpHandler, HandlerResult};
use super::lowlevel::{read_to_vec, ReadResult};

//  We don't do generic HTTP, so we only need these specific methods
#[derive(Show)]
pub enum Method {
    Get,
    Unknown,
}


#[derive(Show)]
pub enum Version {
    Http10,
    Http11,
}

pub enum TransferEncoding {
    Identity,
    Chunked,
}

#[derive(Show)]
pub enum HTTPError {
    BadRequest(&'static str),
    MethodNotAllowed,
}


#[derive(Show)]
pub struct RequestLine<'a>(Method, &'a str, Version);

pub struct Stream {
    fd: Fd,
    handler: HttpHandler,
    buf: Vec<u8>,
}

#[derive(Show)]
pub struct Request<'a> {
    request_line: RequestLine<'a>,
    //headers: Vec<(&'a str, &'a str)>,
    //transfer_encoding: TransferEncoding,
    //content_length: Option<usize>,
}

impl Stream {
    pub fn new(fd: Fd, handler: HttpHandler) -> Stream {
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
                    return HandlerResult::Proceed;
                }
                for i in range(check_start, end - 3) {
                    if self.buf.slice(i, i+4) == b"\r\n\r\n" {
                        match self.parse_request(self.buf.slice(0, i)) {
                            Ok(req) => {
                                (self.handler)(&req);
                                unimplemented!();
                            }
                            Err(e) => {
                                // TODO(tailhook) implement HTTP errors
                                info!("Error parsing request: {:?}", e);
                                return HandlerResult::Remove(self.fd);
                            }
                        }
                    }
                }
                HandlerResult::Proceed
            }
            ReadResult::Fatal(err) => {
                error!("Error handling connection (fd: {}): {:?}",
                    self.fd, err);
                HandlerResult::Remove(self.fd)
            }
            ReadResult::NoData => HandlerResult::Proceed,
            ReadResult::Closed => HandlerResult::Remove(self.fd),
        }
    }

    pub fn _request_line<'x>(&self, chunk: &'x str)
        -> Result<RequestLine<'x>, HTTPError>
    {
        let mut pieces = chunk.trim().words();
        let meth = match pieces.next().unwrap() {
            "GET" => Method::Get,
            _ => return Err(HTTPError::MethodNotAllowed),
        };
        let uri = try!(pieces.next()
            .ok_or(HTTPError::BadRequest("No URI specified")));
        let ver = match pieces.next() {
            Some("HTTP/1.0") => Version::Http10,
            Some("HTTP/1.1") => Version::Http11,
            _ => return Err(HTTPError::BadRequest("Bad HTTP version")),
        };
        Ok(RequestLine(meth, uri, ver))
    }

    pub fn parse_request<'x>(&self, chunk: &'x [u8])
        -> Result<Request<'x>, HTTPError>
    {
        let headers = try!(from_utf8(chunk)
            .map_err(|_| HTTPError::BadRequest("Can't decode headers")));
        let mut lines = headers.split_str("\r\n");
        let request_line = try!(self._request_line(lines.next().unwrap()));
        //let header_name = None;
        //let header_value = "".to_string();
        for line in lines {
            if line.starts_with(" ") || line.starts_with("\t") {
                // continuation
            } else {
                // new header
            }
        }
        return Ok(Request {
            request_line: request_line,
        });
    }
}
