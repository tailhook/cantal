use futures::future::{FutureResult, ok, err};
use tk_http::websocket::{self, Frame};
use serde_json::from_str;

use incoming::Connection;


#[derive(Debug, Serialize, Deserialize)]
#[serde(tag="type", content="payload", rename_all="snake_case")]
pub enum ProtocolElement {
    ConnectionInit,
}


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
                let value: ProtocolElement = match from_str(txt) {
                    Ok(val) => val,
                    Err(e) => {
                        error!("invalid data {:?}: {}", txt, e);
                        return err(websocket::Error::custom("invalid frame"));
                    }
                };
                println!("Value {:?}", value);
            },
            _ => {
                error!("Bad frame received: {:?}", frame);
            }
        }
        ok(())
    }
}
