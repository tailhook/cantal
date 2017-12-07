use std::sync::{Arc, RwLock};
use std::collections::HashMap;

use probor;

use stats::Stats;
use frontend::{Request};
use frontend::routing::Format;
use frontend::quick_reply::{read_json_old, respond_probor};
use query::{Rule, Dataset, query_history};

#[derive(RustcDecodable)]
struct Query {
    rules: HashMap<String, Rule>,
}

struct Response {
    values: HashMap<String, Dataset>,
}

impl<'x> probor::Encodable for Response {
    fn encode<W: probor::Output>(&self, e: &mut probor::Encoder<W>)
        -> Result<(), probor::EncodeError>
    {
        probor_enc_struct!(e, self, {
            values => (),
        });
        Ok(())
    }
}

pub fn serve<S: 'static>(stats: &Arc<RwLock<Stats>>, _format: Format)
    -> Request<S>
{
    let stats = stats.clone();
    read_json_old(move |input: Query, e| {
        let stats: &Stats = &*stats.read().expect("stats not poisoned");
        let ref h = stats.history;
        Box::new(respond_probor(e,
            &Response {
                values: input.rules.into_iter()
                    .map(|(key, rule)| (key, query_history(&rule, h)))
                    .collect(),
            }))
    })
}
