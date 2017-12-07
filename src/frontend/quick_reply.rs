use std::io::BufWriter;
use std::str::from_utf8;

use futures::Async;
use futures::future::{ok, FutureResult};
use tk_http::Status;
use tk_http::server as http;
use tk_http::server::{Error, Codec, RecvMode, Encoder, EncoderDone};
use serde::de::DeserializeOwned;
use serde::ser::Serialize;
use serde_json::{from_slice, to_writer};
use rustc_serialize::json::decode;
use rustc_serialize::Decodable;
use probor::{self, Encodable};

use frontend::{Request, Reply};
use frontend::routing::Format;


pub struct QuickReply<F> {
    inner: Option<F>,
}

pub struct ReadJson<F, S> {
    inner: Option<F>,
    input: Option<S>,
}

pub struct ReadJsonOld<F, S> {
    inner: Option<F>,
    input: Option<S>,
}


pub fn reply<F, S: 'static>(f: F)
    -> Request<S>
    where F: FnOnce(Encoder<S>) -> Reply<S> + 'static,
{
    Box::new(QuickReply {
        inner: Some(f),
    })
}

impl<F, S> Codec<S> for QuickReply<F>
    where F: FnOnce(Encoder<S>) -> Reply<S>,
{
    type ResponseFuture = Reply<S>;
    fn recv_mode(&mut self) -> RecvMode {
        RecvMode::buffered_upfront(0)
    }
    fn data_received(&mut self, data: &[u8], end: bool)
        -> Result<Async<usize>, Error>
    {
        assert!(end);
        assert!(data.len() == 0);
        Ok(Async::Ready(0))
    }
    fn start_response(&mut self, e: http::Encoder<S>) -> Reply<S> {
        let func = self.inner.take().expect("quick reply called only once");
        func(e)
    }
}

pub fn read_json_old<F, S: 'static, V: Decodable + 'static>(f: F)
    -> Request<S>
    where F: FnOnce(V, Encoder<S>) -> Reply<S> + 'static,
{
    Box::new(ReadJsonOld {
        inner: Some(f),
        input: None,
    })
}

impl<F, S, V: Decodable> Codec<S> for ReadJsonOld<F, V>
    where F: FnOnce(V, Encoder<S>) -> Reply<S> + 'static,
{
    type ResponseFuture = Reply<S>;
    fn recv_mode(&mut self) -> RecvMode {
        RecvMode::buffered_upfront(65536)
    }
    fn data_received(&mut self, data: &[u8], end: bool)
        -> Result<Async<usize>, Error>
    {
        assert!(end);
        let data = match from_utf8(data) {
            Ok(x) => x,
            Err(e) => {
                warn!("Invalid utf-8: {}", e);
                return Err(Error::custom(e));
            }
        };
        self.input = match decode(data) {
            Ok(x) => Some(x),
            Err(e) => {
                warn!("Invalid json: {}", e);
                return Err(Error::custom(e));
            }
        };
        Ok(Async::Ready(data.len()))
    }
    fn start_response(&mut self, e: http::Encoder<S>) -> Reply<S> {
        let func = self.inner.take().expect("quick reply called only once");
        let input = self.input.take().expect("quick reply called only once");
        func(input, e)
    }
}

pub fn respond<D: Serialize, S>(mut e: Encoder<S>, format: Format, data: D)
    -> FutureResult<EncoderDone<S>, Error>
{
    e.status(Status::Ok);
    e.add_chunked().unwrap();
    let ctype = match format {
        Format::Json => "application/json",
        Format::Gron => "text/x-gron",
        Format::Cbor => "application/cbor",
    };
    e.add_header("Content-Type", ctype.as_bytes()).unwrap();
    if e.done_headers().unwrap() {
        match format {
            Format::Json => {
                to_writer(&mut BufWriter::new(&mut e), &data)
                    .expect("data is always serializable");
            }
            Format::Gron => {
                unimplemented!();
                /*
                json_to_gron(&mut BufWriter::new(&mut e), "json",
                    &to_value(data).expect("data is always convertible"))
                    .expect("data is always serializable");
                */
            }
            Format::Cbor => {
                unimplemented!();
            }
        };
    }
    ok(e.done())
}

pub fn respond_probor<D: Encodable, S>(mut e: Encoder<S>, data: D)
    -> FutureResult<EncoderDone<S>, Error>
{
    let data = probor::to_buf(&data);
    e.status(Status::Ok);
    e.add_length(data.len() as u64).unwrap();
    e.add_header("Content-Type", b"application/cbor").unwrap();
    if e.done_headers().unwrap() {
        e.write_body(&data);
    }
    ok(e.done())
}

pub fn read_json<F, S: 'static, V: DeserializeOwned + 'static>(f: F)
    -> Request<S>
    where F: FnOnce(V, Encoder<S>) -> Reply<S> + 'static,
{
    Box::new(ReadJson {
        inner: Some(f),
        input: None,
    })
}

impl<F, S, V: DeserializeOwned> Codec<S> for ReadJson<F, V>
    where F: FnOnce(V, Encoder<S>) -> Reply<S> + 'static,
{
    type ResponseFuture = Reply<S>;
    fn recv_mode(&mut self) -> RecvMode {
        RecvMode::buffered_upfront(65536)
    }
    fn data_received(&mut self, data: &[u8], end: bool)
        -> Result<Async<usize>, Error>
    {
        assert!(end);
        self.input = match from_slice(data) {
            Ok(x) => Some(x),
            Err(e) => {
                warn!("Invalid json: {}", e);
                return Err(Error::custom(e));
            }
        };
        Ok(Async::Ready(data.len()))
    }
    fn start_response(&mut self, e: http::Encoder<S>) -> Reply<S> {
        let func = self.inner.take().expect("quick reply called only once");
        let input = self.input.take().expect("quick reply called only once");
        func(input, e)
    }
}
