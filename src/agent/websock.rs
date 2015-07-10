use std::iter::repeat;
use super::http;
use super::http::{Request, BadRequest};
use super::util::Consume;
use super::server::{Context};

use unicase::UniCase;
use byteorder::{BigEndian, ByteOrder};
use hyper::header::{Upgrade, ProtocolName};
use hyper::header::{Connection};
use hyper::version::HttpVersion as Version;
use hyper::header::ConnectionOption::ConnectionHeader;
use websocket::header::{WebSocketVersion, WebSocketKey};


pub fn respond_websock(req: &Request, _context: &mut Context)
    -> Result<http::Response, Box<http::Error>>
{
    if req.version != Version::Http11 {
        return Err(BadRequest::err("Unsupported request HTTP version"));
    }

    if req.headers.get() != Some(&(WebSocketVersion::WebSocket13)) {
        return Err(BadRequest::err("Unsupported WebSocket version"));
    }

    let key  = match req.headers.get::<WebSocketKey>() {
        Some(key) => key,
        None => {
            return Err(BadRequest::err("Missing Sec-WebSocket-Key"));
        }
    };

    match req.headers.get() {
        Some(&Upgrade(ref upgrade)) => {
            let mut correct_upgrade = false;
            for u in upgrade {
                if u.name == ProtocolName::WebSocket {
                    correct_upgrade = true;
                }
            }
            if !correct_upgrade {
                return Err(BadRequest::err(
                    "Invalid Upgrade WebSocket header"));
            }
        }
        None => {
            return Err(BadRequest::err("Missing Upgrade header"));
        }
    };

    match req.headers.get() {
        Some(&Connection(ref connection)) => {
            if !connection.contains(&(ConnectionHeader(
                UniCase("Upgrade".to_string()))))
            {
                return Err(BadRequest::err(
                    "Invalid Connection WebSocket header"));
            }
        }
        None => {
            return Err(BadRequest::err(
                "Missing Connection WebSocket header"));
        }
    }

    Ok(http::Response::accept_websock(key))
}

pub fn parse_message(buf: &mut Vec<u8>, context: &mut Context) {
    if buf.len() < 2 {
        return;
    }
    let fin = buf[0] & 0b10000000 != 0;
    let opcode = buf[0] & 0b00001111;
    let mask = buf[1] & 0b10000000 != 0;
    let mut ln = (buf[1] & 0b01111111) as usize;
    let mut pref = 2;
    if ln == 126 {
        if buf.len() < 4 {
            return;
        }
        ln = BigEndian::read_u16(&buf[2..4]) as usize;
        pref = 4;
    } else if ln == 127 {
        if buf.len() < 10 {
            return
        }
        ln = BigEndian::read_u64(&buf[2..10]) as usize;
        pref = 10;
    }
    if buf.len() < pref + ln + (if mask { 4 } else { 0 }) {
        return;
    }
    if mask {
        let mask = buf[pref..pref+4].to_vec(); // TODO(tailhook) optimize
        pref += 4;
        for (m, t) in mask.iter().cycle().zip(buf[pref..pref+ln].iter_mut()) {
            *t ^= *m;
        }
    }
    {
        let msg = &buf[pref..pref+ln];
        println!("Message {}, {}, {}, len: {}, {:?}", fin, mask, opcode, ln,
            ::std::str::from_utf8(msg));
    }
    buf.consume(pref + ln);
}

pub fn write_text(buf: &mut Vec<u8>, chunk: &str) {
    let bytes = chunk.as_bytes();
    buf.push(0b10000001);  // text message
    if bytes.len() > 65535 {
        buf.push(127);
        let start = buf.len();
        buf.extend(repeat(0).take(8));
        BigEndian::write_u64(&mut buf[start ..],
                             bytes.len() as u64);
    } else if bytes.len() > 125 {
        buf.push(126);
        let start = buf.len();
        buf.extend(repeat(0).take(2));
        BigEndian::write_u16(&mut buf[start ..],
                             bytes.len() as u16);
    } else {
        buf.push(bytes.len() as u8);
    }
    buf.extend(bytes.iter().cloned());
}
