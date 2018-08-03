use std::i32;
use std::collections::BTreeMap;

use juniper::{InputValue};
use juniper::{Value};

use frontend::graphql::{ContextRef, Timestamp};
use history;
use cantal::Value as TipValue;

#[derive(GraphQLInputObject, Debug, Clone)]
#[graphql(name="LastValuesFilter",
          description="Filter for last of value metric")]
pub struct Filter {
    exact_key: Option<Vec<Pair>>,
}

#[derive(Debug, Clone)]
pub struct Key(history::Key);

#[derive(GraphQLInputObject, Debug, Clone)]
pub struct Pair {
    key: String,
    value: String,
}

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
    timestamp: Timestamp,
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
    field timestamp() -> &Timestamp { &self.timestamp }
    field value() -> &String { &self.value }
});

graphql_scalar!(Key {
    description: "A metric key"

    resolve(&self) -> Value {
        Value::object(self.0.as_pairs()
            .map(|(k, v)| (k, v.into())).collect())
    }
    from_input_value(v: &InputValue) -> Option<Key> {
        // pairs must be sorted
        let mut pairs = BTreeMap::new();
        match v {
            InputValue::Object(obj) => {
                for (key, value) in obj {
                    match value.item {
                        InputValue::String(ref value) => {
                            pairs.insert(&key.item[..], &value[..]);
                        }
                        _ => return None,
                    }
                }
            }
            _ => return None,
        }
        return Some(Key(history::Key::unsafe_from_iter(pairs.into_iter())))
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
    if let Some(ref key) = filter.exact_key {

        // pairs must be sorted
        let mut pairs = BTreeMap::new();
        for item in key {
            pairs.insert(&item.key[..], &item.value[..]);
        }
        let key = history::Key::unsafe_from_iter(pairs.into_iter());

        let metric = ctx.stats.history.tip.values.get(&key).map(|(_, met)| {
            match met {
                TipValue::Counter(v) => Metric::Counter(CounterMetric {
                    key: Key(key),
                    value: *v,
                }),
                TipValue::Integer(v) => Metric::Integer(IntegerMetric {
                    key: Key(key),
                    value: *v,
                }),
                TipValue::Float(v) => Metric::Float(FloatMetric {
                    key: Key(key),
                    value: *v,
                }),
                TipValue::State(v) => Metric::State(StateMetric {
                    key: Key(key),
                    timestamp: Timestamp::from_ms(v.0),
                    value: v.1.clone(),
                }),
            }
        });
        // Option<Metric> to 1 element or 0 element vector
        metric.into_iter().collect()
    } else {
        Vec::new()
    }
}
