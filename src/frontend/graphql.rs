use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::time::{SystemTime, UNIX_EPOCH};

use gossip::Gossip;
use juniper::{self, InputValue, RootNode, FieldError, execute};
use juniper::{Value, ExecutionError};
use self_meter_http::{Meter};
use serde_json::{Value as Json, to_value};
use tk_http::Status;

use remote::Remote;
use time_util::duration_to_millis;
use stats::Stats;
use gossip::Peer;
use frontend::{Request};
use frontend::routing::Format;
use frontend::quick_reply::{read_json, respond, respond_status};
use frontend::{status, cgroups, processes};


pub struct ContextRef<'a> {
    pub stats: &'a Stats,
    pub meter: &'a Meter,
    pub gossip: &'a Gossip,
    pub remote: &'a Remote,
}

#[derive(Clone, Debug)]
pub struct Context {
    pub stats: Arc<RwLock<Stats>>,
    pub meter: Meter,
    pub gossip: Gossip,
    pub remote: Remote,
}

pub type Schema<'a> = RootNode<'a, &'a Query, &'a Mutation>;

pub struct Query;
pub struct Local<'a>(PhantomData<&'a ()>);
pub struct Mutation;

#[derive(Deserialize, Clone, Debug)]
pub struct Input {
    pub query: String,
    #[serde(default, rename="operationName")]
    pub operation_name: Option<String>,
    #[serde(default)]
    pub variables: Option<HashMap<String, InputValue>>,
}

#[derive(Debug, Serialize)]
pub struct Output {
    #[serde(skip_serializing_if="Option::is_none")]
    pub data: Option<Value>,
    #[serde(skip_serializing_if="ErrorWrapper::is_empty")]
    pub errors: ErrorWrapper,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum ErrorWrapper {
    Execution(Vec<ExecutionError>),
    Fatal(Json),
}

#[derive(Debug, Serialize, GraphQLObject)]
pub struct Okay {
    ok: bool,
}

pub struct Timestamp(pub SystemTime);

graphql_object!(<'a> Local<'a>: ContextRef<'a> as "Local" |&self| {
    field cgroups(&executor, filter: Option<cgroups::Filter>)
        -> Vec<cgroups::CGroup>
    {
        cgroups::cgroups(executor.context(), filter)
    }
    field processes(&executor, filter: Option<processes::Filter>)
        -> Vec<&processes::Process>
    {
        processes::processes(executor.context(), filter)
    }
});

graphql_object!(<'a> &'a Query: ContextRef<'a> as "Query" |&self| {
    field status(&executor) -> Result<status::GData, FieldError> {
        status::graph(executor.context())
    }
    field local(&executor) -> Local<'a> {
        Local(PhantomData)
    }
    field peers(&executor) -> Vec<Arc<Peer>> {
        executor.context().gossip.get_peers()
    }
});

graphql_object!(<'a> &'a Mutation: ContextRef<'a> as "Mutation" |&self| {
    field noop(&executor) -> Result<Okay, FieldError> {
        Ok(Okay { ok: true })
    }
});

graphql_scalar!(Timestamp {
    description: "A timestamp transferred as a number of milliseconds"

    resolve(&self) -> Value {
        Value::float(duration_to_millis(self.0.duration_since(UNIX_EPOCH)
            .expect("time always in future"))
            as f64)
    }

    from_input_value(_v: &InputValue) -> Option<Timestamp> {
        unimplemented!();
    }
});

pub fn serve<S: 'static>(context: &Context, format: Format)
    -> Request<S>
{
    let ctx = context.clone();
    read_json(move |input: Input, e| {
        let stats: &Stats = &*ctx.stats.read().expect("stats not poisoned");
        let context = ContextRef {
            stats,
            meter: &ctx.meter,
            gossip: &ctx.gossip,
            remote: &ctx.remote,
        };

        let variables = input.variables.unwrap_or_else(HashMap::new);

        let result = execute(&input.query,
            input.operation_name.as_ref().map(|x| &x[..]),
            &Schema::new(&Query, &Mutation),
            &variables,
            &context);
        let out = match result {
            Ok((data, errors)) => {
                Output {
                    data: Some(data),
                    errors: ErrorWrapper::Execution(errors),
                }
            }
            Err(e) => {
                Output {
                    data: None,
                    errors: ErrorWrapper::Fatal(
                        to_value(&e).expect("can serialize error")),
                }
            }
        };

        if out.data.is_some() {
            Box::new(respond(e, format, out))
        } else {
            Box::new(respond_status(Status::BadRequest, e, format, out))
        }
    })
}

pub fn ws_response<'a>(context: &Context, input: &'a Input) -> Output {
    let stats: &Stats = &*context.stats.read().expect("stats not poisoned");
    let context = ContextRef {
        stats,
        meter: &context.meter,
        gossip: &context.gossip,
        remote: &context.remote,
    };

    let empty = HashMap::new();
    let variables = input.variables.as_ref().unwrap_or(&empty);

    let result = execute(&input.query,
        input.operation_name.as_ref().map(|x| &x[..]),
        &Schema::new(&Query, &Mutation),
        &variables,
        &context);

    match result {
        Ok((data, errors)) => {
            Output {
                data: Some(data),
                errors: ErrorWrapper::Execution(errors),
            }
        }
        Err(e) => {
            Output {
                data: None,
                errors: ErrorWrapper::Fatal(
                    to_value(&e).expect("can serialize error")),
            }
        }
    }
}

impl ErrorWrapper {
    fn is_empty(&self) -> bool {
        use self::ErrorWrapper::*;
        match self {
            Execution(v) => v.is_empty(),
            Fatal(..) => false,
        }
    }
}

impl<'a> juniper::Context for ContextRef<'a> {}
