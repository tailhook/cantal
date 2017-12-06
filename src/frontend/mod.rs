mod routing;
mod disk;
mod error_page;
mod quick_reply;

use futures::Future;
use tokio_io::AsyncWrite;
use tk_http::Status;
use tk_http::server::{Codec as CodecTrait, Dispatcher as DispatcherTrait};
use tk_http::server::{Error, Head, EncoderDone};

use frontend::routing::{route, Route};
pub use frontend::quick_reply::{reply, read_json};
pub use frontend::error_page::serve_error_page;


pub type Request<S> = Box<CodecTrait<S, ResponseFuture=Reply<S>>>;
pub type Reply<S> = Box<Future<Item=EncoderDone<S>, Error=Error>>;


pub struct Dispatcher {
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
                serve_error_page(Status::NotFound)
            }
        }
    }
}
