use futures::Async;
use tk_http::server::{Error, Codec, RecvMode, Encoder};
use tk_http::server as http;
use serde::de::DeserializeOwned;
use serde_json::from_slice;

use frontend::{Request, Reply};


pub struct QuickReply<F> {
    inner: Option<F>,
}

pub struct ReadJson<F, S> {
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
