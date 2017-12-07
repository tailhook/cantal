mod disk;
mod error_page;
mod query;
mod quick_reply;
mod routing;
mod status;

use std::sync::{Arc, RwLock};

use futures::Future;
use tokio_io::AsyncWrite;
use tk_http::{Status as Http};
use tk_http::server::{Codec as CodecTrait, Dispatcher as DispatcherTrait};
use tk_http::server::{Error, Head, EncoderDone};
use self_meter_http::Meter;

use stats::Stats;
use frontend::routing::{route, Route};
pub use frontend::quick_reply::{reply, read_json};
pub use frontend::error_page::serve_error_page;


pub type Request<S> = Box<CodecTrait<S, ResponseFuture=Reply<S>>>;
pub type Reply<S> = Box<Future<Item=EncoderDone<S>, Error=Error>>;


pub struct Dispatcher {
    pub meter: Meter,
    pub stats: Arc<RwLock<Stats>>,
}


impl<S: AsyncWrite + Send + 'static> DispatcherTrait<S> for Dispatcher {
    type Codec = Request<S>;
    fn headers_received(&mut self, headers: &Head)
        -> Result<Self::Codec, Error>
    {
        use self::Route::*;
        match route(headers) {
            Index => {
                disk::index_response(headers)
            }
            Static(path) => {
                disk::common_response(headers, path)
            }
            NotFound => {
                serve_error_page(Http::NotFound)
            }
            WebSocket => {
                serve_error_page(Http::NotImplemented)
            }
            Status(format) => {
                Ok(status::serve(&self.meter, &self.stats, format))
            }
            AllProcesses(_) => {
                serve_error_page(Http::NotImplemented)
            }
            AllSockets(_) => {
                serve_error_page(Http::NotImplemented)
            }
            AllMetrics(_) => {
                serve_error_page(Http::NotImplemented)
            }
            AllPeers(_) => {
                serve_error_page(Http::NotImplemented)
            }
            PeersWithRemote(_) => {
                serve_error_page(Http::NotImplemented)
            }
            RemoteStats(_) => {
                serve_error_page(Http::NotImplemented)
            }
            StartRemote(_) => {  // POST
                serve_error_page(Http::NotImplemented)
            }
            Query(format) => {        // POST
                Ok(query::serve(&self.stats, format))
            }
            Remote(_, _) => {
                serve_error_page(Http::NotImplemented)
            }
        }
    }
}
