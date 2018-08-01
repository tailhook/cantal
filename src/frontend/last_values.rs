use std::i32;
use juniper::Value;

use frontend::graphql::ContextRef;
use history;

#[derive(GraphQLInputObject, Debug, Clone)]
#[graphql(name="LastValuesFilter",
          description="Filter for last of value metric")]
pub struct Filter {
    exact_key: Option<Key>,
}

#[derive(Debug, Clone)]
pub struct Key(history::Key);

#[derive(Debug, Clone)]
pub enum Metric {
    Integer(IntegerMetric),
    Counter(CounterMetric),
    Float(FloatMetric),
    State(StateMetric),
}

#[derive(Debug, Clone)]
pub struct IntegerMetric {
    key: Key,
    value: i64,
}

#[derive(Debug, Clone)]
pub struct CounterMetric {
    key: Key,
    value: u64,
}

#[derive(Debug, Clone)]
pub struct FloatMetric {
    key: Key,
    value: f64,
}

#[derive(Debug, Clone)]
pub struct StateMetric {
    key: Key,
    value: String,
}

graphql_object!(IntegerMetric: () |&self| {
    field key() -> &Key { &self.key }
    field as_small_int() -> Option<i32> {
        if self.value <= i32::MAX as i64 && self.value >= i32::MIN as i64 {
            Some(self.value as i32)
        } else {
            None
        }
    }
    field as_float() -> f64 { self.value as f64 }
});

graphql_object!(CounterMetric: () |&self| {
    field key() -> &Key { &self.key }
    field as_small_int() -> Option<i32> {
        if self.value <= i32::MAX as u64 {
            Some(self.value as i32)
        } else {
            None
        }
    }
    field as_float() -> f64 { self.value as f64 }
});

graphql_object!(FloatMetric: () |&self| {
    field key() -> &Key { &self.key }
    field value() -> f64 { self.value }
});

graphql_object!(StateMetric: () |&self| {
    field key() -> &Key { &self.key }
    field value() -> &String { &self.value }
});

graphql_scalar!(Key {
    description: "A metric key (dict of string: string)"

    resolve(&self) -> Value {
        Value::object(self.0.as_pairs()
            .map(|(k, v)| (k, v.into())).collect())
    }

    from_input_value(_v: &InputValue) -> Option<Key> {
        unimplemented!();
    }
});

graphql_interface!(Metric: () |&self| {
    field key() -> &Key {
        use self::Metric::*;
        match *self {
            | Integer(IntegerMetric { ref key, .. })
            | Counter(CounterMetric { ref key, .. })
            | Float(FloatMetric { ref key, .. })
            | State(StateMetric { ref key, .. })
            => key,
        }
    }

    instance_resolvers: |_| {
        &IntegerMetric => maybe!(m if let Metric::Integer(ref m) = *self),
        &CounterMetric => maybe!(m if let Metric::Counter(ref m) = *self),
        &FloatMetric => maybe!(m if let Metric::Float(ref m) = *self),
        &StateMetric => maybe!(m if let Metric::State(ref m) = *self),
    }
});

pub fn query<'x>(ctx: &ContextRef<'x>, filter: Filter)
    -> Vec<Metric>
{
    unimplemented!();
}
