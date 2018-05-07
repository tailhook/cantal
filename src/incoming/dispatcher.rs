use juniper::{Value, ExecutionError};
use futures::future::{FutureResult, ok, err};
use tk_http::websocket::{self, Frame, Packet};
use serde_json::{from_str, to_string};
use graphql_parser::parse_query;
use graphql_parser::query::OperationDefinition::{Subscription};
use graphql_parser::query::{Definition, Document};
use graphql_parser::query::{Selection};

use incoming::{Connection, Incoming};
use incoming::{subscription_to_query};
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
    Data { id: String, payload: Output },
}

#[derive(Debug, Serialize)]
pub struct Output {
    pub data: Value,
    pub errors: Vec<ExecutionError>,
}


pub struct Dispatcher {
    pub conn: Connection,
    pub graphql: graphql::Context,
    pub incoming: Incoming,
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
                        self.conn.0.tx.unbounded_send(packet)
                            .map_err(|e| trace!("can't reply with ack: {}", e))
                            .ok();
                    }
                    InputMessage::Start {id, payload} => {
                        start_query(id, payload, &self.conn,
                            &self.graphql, &self.incoming);
                    }
                    InputMessage::Stop {id} => {
                        self.incoming.unsubscribe_status(&self.conn, &id);
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

fn has_subscription(doc: &Document) -> bool {
    for d in &doc.definitions {
        match *d {
            Definition::Operation(Subscription(_)) => {
                return true;
            }
            _ => {}
        }
    }
    return false;
}

fn start_query(id: String, payload: graphql::Input,
    conn: &Connection, context: &graphql::Context, incoming: &Incoming)
{
    let q = parse_query(&payload.query)
        .expect("Request is good"); // TODO(tailhook)
    if has_subscription(&q) {
        // TODO(tailhook) optimize this deep clone
        let qq = subscription_to_query(q.clone());
        let input = graphql::Input {
            query: qq.to_string(),
            ..payload
        };
        for d in &q.definitions {
            match *d {
                Definition::Operation(Subscription(ref sub)) => {
                    for item in &sub.selection_set.items {
                        match *item {
                            Selection::Field(ref f) if f.name == "status" => {
                                incoming.subscribe_status(conn,
                                    &id, &input);
                            }
                            // TODO(tailhook) maybe validate?
                            // For now invalid fields will error in juniper
                            // executor.
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }

        let result = graphql::ws_response(context, &input);
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
        conn.0.tx.unbounded_send(packet)
            .map_err(|e| {
                trace!("can't reply with ack: {}", e)
            }).ok();
    } else {
        let result = graphql::ws_response(context, &payload);
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
        conn.0.tx.unbounded_send(packet)
            .map_err(|e| {
                trace!("can't reply with ack: {}", e)
            }).ok();
    }
}
