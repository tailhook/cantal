use juniper::{Value, ExecutionError};
use futures::future::{FutureResult, ok, err};
use tk_http::websocket::{self, Frame, Packet};
use serde_json::{from_str, to_string};

use incoming::Connection;
use frontend::graphql;


#[derive(Debug, Serialize, Deserialize)]
pub struct ConnectionParams {
}

#[derive(Debug, Deserialize)]
#[serde(tag="type", rename_all="snake_case")]
pub enum InputMessage {
    ConnectionInit { payload: ConnectionParams },
    Start { payload: graphql::Input, id: String },
    Stop { id: String },
}
#[derive(Debug, Serialize)]
#[serde(tag="type", rename_all="snake_case")]
pub enum OutputMessage {
    ConnectionAck,
    ConnectionKeepAlive,
    Data { id: String, payload: Output },
}

#[derive(Debug, Serialize)]
pub struct Output {
    data: Value,
    errors: Vec<ExecutionError>,
}


pub struct Dispatcher {
    pub conn: Connection,
    pub graphql: graphql::Context,
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
                    InputMessage::ConnectionInit { payload: _payload } => {
                        let packet = Packet::Text(
                            to_string(&OutputMessage::ConnectionAck)
                            .expect("can serialize"));
                        self.conn.tx.unbounded_send(packet)
                            .map_err(|e| trace!("can't reply with ack: {}", e))
                            .ok();
                    }
                    InputMessage::Start {id, payload} => {
                        let result = graphql::ws_response(
                            &self.graphql, &payload);
                        let packet = Packet::Text(
                            to_string(&OutputMessage::Data {
                                id: id,
                                payload: match result {
                                    Ok((data, errors))
                                    => Output { data, errors },
                                    Err(e) => {
                                        info!("Request error {:?}", e);
                                        unimplemented!();
                                    }
                                },
                            })
                            .expect("can serialize"));
                        self.conn.tx.unbounded_send(packet)
                            .map_err(|e| {
                                trace!("can't reply with ack: {}", e)
                            }).ok();
                    }
                    InputMessage::Stop {id: _} => {
                        // TODO(tailhook) unsubscribe
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
