use super::http;
use super::http::{Request, BadRequest};
use super::server::Context;

use unicase::UniCase;
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
