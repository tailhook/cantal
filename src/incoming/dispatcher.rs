use futures::future::{FutureResult, ok, err};
use tk_http::websocket::{self, Frame, Packet};
use serde_json::{from_str, to_string};

use incoming::Connection;


#[derive(Debug, Serialize, Deserialize)]
pub struct ConnectionParams {
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag="type", content="payload", rename_all="snake_case")]
pub enum InputMessage {
    ConnectionInit(ConnectionParams),
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag="type", content="payload", rename_all="snake_case")]
pub enum OutputMessage {
    ConnectionAck,
    ConnectionKeepAlive,
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
            }
            Frame::Text(txt) => {
                let value = match from_str(txt) {
                    Ok(val) => val,
                    Err(e) => {
                        error!("invalid data {:?}: {}", txt, e);
                        return err(websocket::Error::custom("invalid frame"));
                    }
                };
                match value {
                    InputMessage::ConnectionInit(_params) => {
                        let packet = Packet::Text(
                            to_string(&OutputMessage::ConnectionAck)
                            .expect("can serialize"));
                        self.conn.tx.unbounded_send(packet)
                            .map_err(|e| trace!("can't reply with ack: {}", e))
                            .ok();
                    }
                }
            }
            Frame::Close(code, reason) => {
                debug!("Closed, code {}: {:?}", code, reason);
                // TODO(tailhook) should we do anything?
            }
            _ => {
                error!("Bad frame received: {:?}", frame);
            }
        }
        ok(())
    }
}
