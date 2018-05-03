use std::sync::{Arc, RwLock};
use std::collections::HashMap;

use juniper::{InputValue, RootNode, execute};

use stats::Stats;
use frontend::{Request};
use frontend::routing::Format;
use frontend::quick_reply::{read_json, respond};

pub struct Context<'a> {
    stats: &'a Stats,
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
    variables: HashMap<String, InputValue>,
}


graphql_object!(<'a> &'a Query: Context<'a> as "Query" |&self| {
});

graphql_object!(<'a> &'a Mutation: Context<'a> as "Mutation" |&self| {
});

pub fn serve<S: 'static>(stats: &Arc<RwLock<Stats>>, format: Format)
    -> Request<S>
{
    let stats = stats.clone();
    read_json(move |input: Input, e| {
        let stats: &Stats = &*stats.read().expect("stats not poisoned");
        let context = Context {
            stats,
        };

        let result = execute(&input.query,
            input.operation_name.as_ref().map(|x| &x[..]),
            &Schema::new(&Query, &Mutation),
            &input.variables,
            &context);

        Box::new(respond(e, format, result))
    })
}
