//  This implementation is an implementation of subset of HTTP.
//  It *may be unsafe* to expose this implementation to the untrusted internet

use std::os::unix::Fd;
use super::{HttpHandler, HandlerResult};
use super::lowlevel::{read_to_vec, ReadResult};

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
                        unimplemented!();
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
}
