//  This implementation is an implementation of subset of HTTP.
//  It *may be unsafe* to expose this implementation to the untrusted internet

use std::os::unix::Fd;
use super::{HttpHandler, HandlerResult};

const BUFFER_SIZE: usize = 1024;


//  We don't do generic HTTP, so we only need these specific methods
pub enum Method {
    Get,
    Unknown,
}


pub enum Version {
    Http10,
    Http11,
}

pub enum TransferEncoding {
    Identity,
    Chunked,
}


pub struct RequestLine(Method, String, Version);

pub struct Stream {
    fd: Fd,
    handler: HttpHandler,
    buf: Vec<u8>,
}

pub struct Request<'a> {
    headers_buf: &'a [u8],
    request_line: RequestLine,
    headers: Vec<(&'a str, &'a str)>,
    transfer_encoding: TransferEncoding,
    content_length: Option<usize>,
}

impl Stream {
    pub fn new(fd: Fd, handler: HttpHandler) -> Stream {
        return Stream {
            fd: fd,
            handler: handler,
            buf: Vec::with_capacity(BUFFER_SIZE),
        }
    }
    pub fn read_http(&mut self) -> HandlerResult {
        unimplemented!();
    }
}
