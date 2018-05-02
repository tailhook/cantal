mod add_host;
mod all_metrics;
mod disk;
mod error_page;
mod processes;
mod query;
mod quick_reply;
mod routing;
mod sockets;
mod status;
mod peers;
mod websocket;

use std::sync::{Arc, RwLock};

use futures::Future;
use gossip::Gossip;
use self_meter_http::Meter;
use tk_http::server::{Codec as CodecTrait, Dispatcher as DispatcherTrait};
use tk_http::server::{Error, Head, EncoderDone};
use tk_http::{Status as Http};
use tokio_io::{AsyncRead, AsyncWrite};

use incoming::Incoming;
use stats::Stats;
use frontend::routing::{route, Route};
pub use frontend::quick_reply::{reply, read_json};
pub use frontend::error_page::serve_error_page;


pub type Request<S> = Box<CodecTrait<S, ResponseFuture=Reply<S>>>;
pub type Reply<S> = Box<Future<Item=EncoderDone<S>, Error=Error>>;


pub struct Dispatcher {
    pub meter: Meter,
    pub stats: Arc<RwLock<Stats>>,
    pub gossip: Gossip,
    pub incoming: Incoming,
}


impl<S> DispatcherTrait<S> for Dispatcher
    where S: AsyncRead + AsyncWrite + Send + 'static
{
    type Codec = Request<S>;
    fn headers_received(&mut self, headers: &Head)
        -> Result<Self::Codec, Error>
    {
        use self::Route::*;
        let up = match headers.get_websocket_upgrade() {
            Ok(up) => up,
            Err(()) => {
                info!("Invalid websocket handshake");
                return serve_error_page(Http::BadRequest);
            }
        };
        match route(headers, up) {
            Index => {
                disk::index_response(headers)
            }
            Static(path) => {
                disk::common_response(headers, path)
            }
            NotFound => {
                serve_error_page(Http::NotFound)
            }
            GraphqlWs(ws) => {
                Ok(websocket::serve(&self.stats, ws, &self.incoming))
            }
            Status(format) => {
                Ok(status::serve(&self.meter, &self.stats, format))
            }
            AllProcesses(format) => {
                Ok(processes::serve(&self.stats, format))
            }
            AllSockets(format) => {
                Ok(sockets::serve(&self.stats, format))
            }
            AllMetrics(_) => {
                Ok(all_metrics::serve(&self.stats))
            }
            AllPeers(format) => {
                Ok(peers::serve(&self.gossip, format))
            }
            PeersWithRemote(format) => {
                Ok(peers::serve_only_remote(&self.gossip, format))
            }
            RemoteStats(_) => {
                serve_error_page(Http::NotImplemented)
            }
            StartRemote(_) => {  // POST
                serve_error_page(Http::NotImplemented)
            }
            Query(format) => {   // POST
                Ok(query::serve(&self.stats, format))
            }
            AddHost(format) => { // POST
                Ok(add_host::add_host(&self.gossip, format))
            }
            Remote(_, _) => {
                serve_error_page(Http::NotImplemented)
            }
        }
    }
}
