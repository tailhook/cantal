use std::fmt;
use std::sync::Arc;
use std::borrow::Borrow;

use juniper::{Value};


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Hostname(Arc<str>);

impl Borrow<str> for Hostname {
    fn borrow(&self) -> &str {
        &*self.0
    }
}

impl AsRef<str> for Hostname {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl From<String> for Hostname {
    fn from(s: String) -> Hostname {
        Hostname(Arc::from(&s[..]))
    }
}

impl<'a> From<&'a String> for Hostname {
    fn from(s: &String) -> Hostname {
        Hostname(Arc::from(&s[..]))
    }
}

impl<'a> From<&'a str> for Hostname {
    fn from(s: &str) -> Hostname {
        Hostname(Arc::from(s))
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

impl fmt::Display for Hostname {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", &self.0[..])
    }
}
