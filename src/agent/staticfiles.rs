use std::fs::File;
use std::path::PathBuf;
use std::path::Component::ParentDir;
use std::io::ErrorKind::{NotFound};
use std::io::Read;
use std::env::current_exe;

use super::aio::http;


pub fn serve(req: &http::Request) -> Result<http::Response, http::Error>
{
    let mut uripath = PathBuf::from(&format!(".{}", req.uri()));
    if req.uri().ends_with("/") {
        uripath = uripath.join("index.html");
    }
    if uripath.components().any(|x| x == ParentDir) {
        return Err(http::Error::BadRequest("The dot-dot in uri path"));
    }
    let mut filename = current_exe().unwrap();
    filename.pop();
    filename.push("public");
    filename = filename.join(&uripath);
    let data = try!(File::open(&filename)
        .map_err(|e| if e.kind() == NotFound {
                http::Error::NotFound
            } else {
                error!("Error opening file for uri {:?}: {}", req.uri(), e);
                http::Error::ServerError("Can't open file")
            })
        .and_then(|mut f| {
            let mut buf = Vec::with_capacity(65536);
            f.read_to_end(&mut buf)
            .map_err(|e| {
                error!("Error reading file for uri {:?}: {}", req.uri(), e);
                http::Error::ServerError("Can't read file")
            })
            .map(|_| buf)
        }));
    // TODO(tailhook) find out mime type
    let mut builder = http::ResponseBuilder::new(req, http::Status::Ok);
    if req.uri().ends_with(".js") {
        builder.add_header(
            "Content-Type: application/javascript; charset=utf-8");
    } else if req.uri().ends_with(".css") {
        builder.add_header(
            "Content-Type: text/css; charset=utf-8");
    }
    builder.set_body(data);
    Ok(builder.take())
}
