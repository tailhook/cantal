use std::fs::File;
use std::path::PathBuf;
use std::path::Component::ParentDir;
use std::io::ErrorKind::NotFound as FileNotFound;
use std::io::Read;
use std::env::current_exe;

use hyper::status::StatusCode;
use hyper::uri::RequestUri::{AbsolutePath};
use super::http::{Error, BadRequest, NotFound, ServerError};
use super::http::{Request, Response};


pub fn serve(req: &Request) -> Result<Response, Box<Error>>
{
    let uri = if let AbsolutePath(ref uri) = req.uri {
        uri
    } else {
        return Err(Box::new(BadRequest("Wrong URI kind")));
    };
    let mut uripath = PathBuf::from(&format!(".{}", uri));
    if uri.ends_with("/") {
        uripath = uripath.join("index.html");
    }
    if uripath.components().any(|x| x == ParentDir) {
        return Err(Box::new(BadRequest("The dot-dot in uri path")));
    }
    let mut filename = current_exe().unwrap();
    filename.pop();
    filename.push("public");
    filename = filename.join(&uripath);
    let data = try!(File::open(&filename)
        .map_err(|e| if e.kind() == FileNotFound {
                Box::new(NotFound) as Box<Error>
            } else {
                error!("Error opening file for uri {:?}: {}", uri, e);
                Box::new(ServerError("Can't open file")) as Box<Error>
            })
        .and_then(|mut f| {
            let mut buf = Vec::with_capacity(65536);
            f.read_to_end(&mut buf)
            .map_err(|e| {
                error!("Error reading file for uri {:?}: {}", uri, e);
                Box::new(ServerError("Can't read file")) as Box<Error>
            })
            .map(|_| buf)
        }));
    if uri == "/" {
        return Ok(Response::static_mime_vec(StatusCode::Ok,
            mime!(Text/Html; Charset=Utf8),
            data));
    } else if uri.ends_with(".js") {
        return Ok(Response::static_mime_vec(StatusCode::Ok,
            mime!(Application/Javascript; Charset=Utf8),
            data));
    } else if uri.ends_with(".css") {
        return Ok(Response::static_mime_vec(StatusCode::Ok,
            mime!(Text/Css; Charset=Utf8),
            data));
    } else {
        return Ok(Response::static_mime_vec(StatusCode::Ok,
            "application/octed-stream".parse().unwrap(),
            data));
    }
}
