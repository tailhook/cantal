use std::path::{PathBuf};
use std::sync::Arc;
use std::env::current_exe;

use futures::{Future, Async};
use futures::future::{ok, FutureResult, Either, loop_fn, Loop};
use futures_cpupool::{CpuPool, CpuFuture};
use tokio_io::AsyncWrite;
use tk_http::server;
use tk_http::Status;
use http_file_headers::{Input, Output, Config};

use frontend::Request;

lazy_static! {
    static ref POOL: CpuPool = CpuPool::new(8);
    static ref CONFIG: Arc<Config> = Config::new()
        .add_index_file("index.html")
        .done();
}

type ResponseFuture<S> = Box<Future<Item=server::EncoderDone<S>,
                                   Error=server::Error>>;

struct Codec {
    fut: Option<CpuFuture<Output, Status>>,
}

fn common_headers<S>(e: &mut server::Encoder<S>) {
    e.format_header("Server",
        format_args!("cantal/{}", env!("CARGO_PKG_VERSION"))).unwrap();
}

fn respond_error<S: 'static>(status: Status, mut e: server::Encoder<S>)
    -> FutureResult<server::EncoderDone<S>, server::Error>
{
    let body = format!("{} {}", status.code(), status.reason());
    e.status(status);
    e.add_length(body.as_bytes().len() as u64).unwrap();
    common_headers(&mut e);
    if e.done_headers().unwrap() {
        e.write_body(body.as_bytes());
    }
    ok(e.done())
}

impl<S: AsyncWrite + Send + 'static> server::Codec<S> for Codec {
    type ResponseFuture = ResponseFuture<S>;
    fn recv_mode(&mut self) -> server::RecvMode {
        server::RecvMode::buffered_upfront(0)
    }
    fn data_received(&mut self, data: &[u8], end: bool)
        -> Result<Async<usize>, server::Error>
    {
        debug_assert!(end && data.len() == 0);
        Ok(Async::Ready(0))
    }
    fn start_response(&mut self, mut e: server::Encoder<S>)
        -> Self::ResponseFuture
    {
        Box::new(self.fut.take().unwrap().then(move |result| {
            match result {
                Ok(Output::File(outf)) | Ok(Output::FileRange(outf)) => {
                    if outf.is_partial() {
                        e.status(Status::PartialContent);
                    } else {
                        e.status(Status::Ok);
                    }
                    e.add_length(outf.content_length()).unwrap();
                    common_headers(&mut e);
                    for (name, val) in outf.headers() {
                        e.format_header(name, val).unwrap();
                    }
                    // add headers
                    if e.done_headers().unwrap() {
                        // start writing body
                        Either::B(loop_fn((e, outf), |(mut e, mut outf)| {
                            POOL.spawn_fn(move || {
                                outf.read_chunk(&mut e).map(|b| (b, e, outf))
                            }).and_then(|(b, e, outf)| {
                                e.wait_flush(4096).map(move |e| (b, e, outf))
                            }).map(|(b, e, outf)| {
                                if b == 0 {
                                    Loop::Break(e.done())
                                } else {
                                    Loop::Continue((e, outf))
                                }
                            }).map_err(|e| server::Error::custom(e))
                        }))
                    } else {
                        Either::A(ok(e.done()))
                    }
                }
                Ok(Output::FileHead(head)) | Ok(Output::NotModified(head)) => {
                    if head.is_not_modified() {
                        e.status(Status::NotModified);
                    } else if head.is_partial() {
                        e.status(Status::PartialContent);
                        e.add_length(head.content_length()).unwrap();
                    } else {
                        e.status(Status::Ok);
                        e.add_length(head.content_length()).unwrap();
                    }
                    common_headers(&mut e);
                    for (name, val) in head.headers() {
                        e.format_header(name, val).unwrap();
                    }
                    assert_eq!(e.done_headers().unwrap(), false);
                    Either::A(ok(e.done()))
                }
                Ok(Output::InvalidRange) => {
                    Either::A(respond_error(
                        Status::RequestRangeNotSatisfiable, e))
                }
                Ok(Output::InvalidMethod) => {
                    Either::A(respond_error(
                        Status::MethodNotAllowed, e))
                }
                Ok(Output::NotFound) | Ok(Output::Directory) => {
                    Either::A(respond_error(Status::NotFound, e))
                }
                Err(status) => {
                    Either::A(respond_error(status, e))
                }
            }
        }))
    }
}

pub fn filepath(tail: &str) -> PathBuf {
    // tail is checked for parent directories in routing
    let mut filename = current_exe().unwrap();
    filename.pop();
    filename.push("public");
    filename.push(tail);
    return filename;
}

pub fn index_response<S>(head: &server::Head)
    -> Result<Request<S>, server::Error>
    where S: AsyncWrite + Send + 'static
{
    let inp = Input::from_headers(&*CONFIG, head.method(), head.headers());
    let path = filepath("index.html");
    let fut = POOL.spawn_fn(move || {
        inp.probe_file(&path).map_err(|e| {
            error!("Error reading file {:?}: {}", path, e);
            Status::InternalServerError
        })
    });
    Ok(Box::new(Codec {
        fut: Some(fut),
    }) as Request<S>)
}

pub fn common_response<S>(head: &server::Head, path: String)
    -> Result<Request<S>, server::Error>
    where S: AsyncWrite + Send + 'static
{
    let inp = Input::from_headers(&*CONFIG, head.method(), head.headers());
    // path is validated for ".." and root in routing
    let path = filepath(&path);
    let fut = POOL.spawn_fn(move || {
        inp.probe_file(&path).map_err(|e| {
            error!("Error reading file {:?}: {}", path, e);
            Status::InternalServerError
        })
    });
    Ok(Box::new(Codec {
        fut: Some(fut),
    }))
}
