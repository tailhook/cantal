use futures::future::{FutureResult, ok};
use tk_http::websocket::{self, Frame};

use incoming::Connection;


pub struct Dispatcher {
    pub conn: Connection,
}

impl websocket::Dispatcher for Dispatcher {
    // TODO(tailhook) implement backpressure
    type Future = FutureResult<(), websocket::Error>;
    fn frame(&mut self, frame: &Frame) -> Self::Future {
        match *frame {
            Frame::Binary(_) => {
                error!("Received binary frame");
            },
            Frame::Text(txt) => {
                println!("TEXT {:?}", txt);
            },
            _ => {
                error!("Bad frame received: {:?}", frame);
            }
        }
        ok(())
    }
}
