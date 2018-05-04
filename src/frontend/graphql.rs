use std::sync::{Arc, RwLock};
use std::collections::HashMap;

use juniper::{InputValue, RootNode, FieldError, execute};
use juniper::{Value, ExecutionError, GraphQLError};
use self_meter_http::{Meter};

use stats::Stats;
use frontend::{Request};
use frontend::routing::Format;
use frontend::quick_reply::{read_json, respond};
use frontend::status;


pub struct ContextRef<'a> {
    pub stats: &'a Stats,
    pub meter: &'a Meter,
}

#[derive(Clone)]
pub struct Context {
    pub stats: Arc<RwLock<Stats>>,
    pub meter: Meter,
}

pub type Schema<'a> = RootNode<'a, &'a Query, &'a Mutation>;

pub struct Query;
pub struct Mutation;

#[derive(Deserialize, Clone, Debug)]
pub struct Input {
    pub query: String,
    #[serde(default, rename="operationName")]
    pub operation_name: Option<String>,
    #[serde(default)]
    pub variables: Option<HashMap<String, InputValue>>,
}


graphql_object!(<'a> &'a Query: ContextRef<'a> as "Query" |&self| {
    field status(&executor) -> Result<status::GData, FieldError> {
        status::graph(executor.context())
    }
});

graphql_object!(<'a> &'a Mutation: ContextRef<'a> as "Mutation" |&self| {
});

pub fn serve<S: 'static>(context: &Context, format: Format)
    -> Request<S>
{
    let stats = context.stats.clone();
    let meter = context.meter.clone();
    read_json(move |input: Input, e| {
        let stats: &Stats = &*stats.read().expect("stats not poisoned");
        let context = ContextRef {
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

pub fn ws_response<'a>(context: &Context, input: &'a Input)
    -> Result<(Value, Vec<ExecutionError>), GraphQLError<'a>>
{
    let stats: &Stats = &*context.stats.read().expect("stats not poisoned");
    let context = ContextRef {
        stats,
        meter: &context.meter,
    };

    let empty = HashMap::new();
    let variables = input.variables.as_ref().unwrap_or(&empty);

    execute(&input.query,
        input.operation_name.as_ref().map(|x| &x[..]),
        &Schema::new(&Query, &Mutation),
        &variables,
        &context)
}
