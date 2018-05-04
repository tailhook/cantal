use juniper::{Value, ExecutionError};
use futures::future::{FutureResult, ok, err};
use tk_http::websocket::{self, Frame, Packet};
use serde_json::{from_str, to_string};
use graphql_parser::parse_query;
use graphql_parser::query::OperationDefinition::{Subscription, Query};
use graphql_parser::query::{Definition, Document, Query as QueryParams};

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
                        start_query(id, payload, &self.conn, &self.graphql);
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

fn subscription_to_query(doc: Document) -> Document {
    let definitions = doc.definitions.into_iter().map(|def| {
        match def {
            Definition::Operation(Subscription(s)) => {
                Definition::Operation(Query(QueryParams {
                    position: s.position,
                    name: s.name,
                    variable_definitions: s.variable_definitions,
                    directives: s.directives,
                    selection_set: s.selection_set,
                }))
            }
            def => def,
        }
    }).collect();
    return Document { definitions }
}

fn start_query(id: String, payload: graphql::Input,
    conn: &Connection, context: &graphql::Context)
{
    let q = parse_query(&payload.query)
        .expect("Request is good"); // TODO(tailhook)
    if has_subscription(&q) {
        let qq = subscription_to_query(q);
        let input = graphql::Input {
            query: qq.to_string(),
            ..payload
        };
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
        conn.tx.unbounded_send(packet)
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
        conn.tx.unbounded_send(packet)
            .map_err(|e| {
                trace!("can't reply with ack: {}", e)
            }).ok();
    }
}
