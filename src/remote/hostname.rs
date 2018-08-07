use std::sync::Arc;
use std::borrow::Borrow;

use juniper::{Value};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hostname(Arc<str>);

impl Borrow<str> for Hostname {
    fn borrow(&self) -> &str {
        &*self.0
    }
}

impl<S: AsRef<str>> From<S> for Hostname {
    fn from(s: S) -> Hostname {
        Hostname(Arc::from(s.as_ref()))
    }
}

graphql_scalar!(Hostname {
    description: "A hostname of a remote host"

    resolve(&self) -> Value {
        Value::string(&self.0)
    }
    from_input_value(v: &InputValue) -> Option<Hostname> {
        v.as_string_value().map(|x| Hostname::from(x))
    }
});
