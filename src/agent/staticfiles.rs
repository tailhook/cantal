use std::fs::File;
use std::io::ErrorKind::{FileNotFound};
use std::io::Read;
use std::os::self_exe_path;

use super::aio::http;


pub fn serve(req: &http::Request) -> Result<http::Response, http::Error>
{
    let mut uripath = Path::new(format!(".{}", req.uri()));
    if req.uri().ends_with("/") {
        uripath = uripath.join("index.html");
    }
    if uripath.components().any(|x| x == b"..") {
        return Err(http::Error::BadRequest("The dot-dot in uri path"));
    }
    let filename = self_exe_path().unwrap().join("public").join(uripath);
    let data = try!(File::open(&filename)
        .map_err(|e| if e.kind() == FileNotFound {
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
    builder.set_body(data);
    Ok(builder.take())
}
