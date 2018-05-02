use std::sync::{Arc, RwLock};

use futures::{Async, Future};
use futures::future::{ok};
use tk_bufstream::{ReadBuf, WriteBuf};
use tk_http::server;
use tk_http::Status;
use tk_http::server::{Encoder, Error, RecvMode};
use tk_http::server::{WebsocketHandshake};
use tk_http::websocket::{ServerCodec};
use tokio_io::{AsyncRead, AsyncWrite};
use tk_easyloop::spawn;


use frontend::{Request, Reply};
use incoming::Incoming;
use stats::Stats;


struct WsCodec {
    ws: WebsocketHandshake,
    incoming: Incoming,
}


impl<S: 'static + AsyncRead + AsyncWrite> server::Codec<S> for WsCodec {
    type ResponseFuture = Reply<S>;
    fn recv_mode(&mut self) -> RecvMode {
        RecvMode::hijack()
    }
    fn data_received(&mut self, data: &[u8], end: bool)
        -> Result<Async<usize>, Error>
    {
        debug_assert!(end);
        debug_assert_eq!(data.len(), 0);
        Ok(Async::Ready(data.len()))
    }
    fn start_response(&mut self, mut e: Encoder<S>) -> Self::ResponseFuture {
        e.status(Status::SwitchingProtocol);
        e.add_date();
        e.add_header("Server",
            concat!("cantal/", env!("CARGO_PKG_VERSION"))
        ).unwrap();
        e.add_header("Connection", "upgrade").unwrap();
        e.add_header("Upgrade", "websocket").unwrap();
        e.format_header("Sec-Websocket-Accept", &self.ws.accept).unwrap();
        e.done_headers().unwrap();
        Box::new(ok(e.done()))
    }
    fn hijack(&mut self, write_buf: WriteBuf<S>, read_buf: ReadBuf<S>){
        let inp = read_buf.framed(ServerCodec);
        let out = write_buf.framed(ServerCodec);
        let (token, fut) = self.incoming.connected(out, inp);
        spawn(fut
            .map_err(|e| debug!("websocket closed: {}", e))
            .then(move |r| {
                drop(token);
                r
            }));
    }
}

pub fn serve<S: 'static>(stats: &Arc<RwLock<Stats>>,
    ws: WebsocketHandshake, incoming: &Incoming)
    -> Request<S>
    where S: AsyncRead + AsyncWrite + 'static,
{
    let _stats = stats.clone();
    Box::new(WsCodec {
        ws,
        incoming: incoming.clone(),
    })
}
