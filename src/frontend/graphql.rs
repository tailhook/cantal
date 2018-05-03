use std::sync::{Arc, RwLock};
use std::collections::HashMap;

use juniper::{InputValue, RootNode, FieldError, execute};
use self_meter_http::{Meter};

use stats::Stats;
use frontend::{Request};
use frontend::routing::Format;
use frontend::quick_reply::{read_json, respond};
use frontend::status;

pub struct Context<'a> {
    pub stats: &'a Stats,
    pub meter: &'a Meter,
}

pub type Schema<'a> = RootNode<'a, &'a Query, &'a Mutation>;

pub struct Query;
pub struct Mutation;

#[derive(Deserialize, Clone, Debug)]
pub struct Input {
    query: String,
    #[serde(default, rename="operationName")]
    operation_name: Option<String>,
    #[serde(default)]
    variables: Option<HashMap<String, InputValue>>,
}


graphql_object!(<'a> &'a Query: Context<'a> as "Query" |&self| {
    field status(&executor) -> Result<status::GData, FieldError> {
        status::graph(executor.context())
    }
});

graphql_object!(<'a> &'a Mutation: Context<'a> as "Mutation" |&self| {
});

pub fn serve<S: 'static>(meter: &Meter, stats: &Arc<RwLock<Stats>>,
    format: Format)
    -> Request<S>
{
    let stats = stats.clone();
    let meter = meter.clone();
    read_json(move |input: Input, e| {
        let stats: &Stats = &*stats.read().expect("stats not poisoned");
        let context = Context {
            stats,
            meter: &meter,
        };

        let variables = input.variables.unwrap_or_else(HashMap::new);

        let result = execute(&input.query,
            input.operation_name.as_ref().map(|x| &x[..]),
            &Schema::new(&Query, &Mutation),
            &variables,
            &context);

        Box::new(respond(e, format, result))
    })
}
