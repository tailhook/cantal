use std::i32;
use std::collections::BTreeMap;

use juniper::{InputValue};
use juniper::{Value};

use frontend::graphql::{ContextRef, Timestamp};
use history;
use remote::Hostname;
use cantal::Value as TipValue;
use incoming::tracking;
use stats::Stats;

use self::RemoteMetric as RM;

#[derive(GraphQLInputObject, Debug, Clone)]
#[graphql(name="LastValuesFilter",
          description="Filter for last of value metric")]
pub struct Filter {
    exact_key: Option<Vec<Pair>>,
}

#[derive(GraphQLInputObject, Debug, Clone)]
#[graphql(name="InternalLastValuesFilter",
          description="Filter for last of value metric (for internal API)")]
pub struct InternalFilter {
    exact_key: Vec<Pair>,
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
pub enum RemoteMetric {
    Integer(RemoteIntegerMetric),
    Counter(RemoteCounterMetric),
    Float(RemoteFloatMetric),
    State(RemoteStateMetric),
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

#[derive(Debug, Clone)]
pub struct RemoteIntegerMetric {
    hostname: Hostname,
    key: Key,
    value: i64,
}

#[derive(Debug, Clone)]
pub struct RemoteCounterMetric {
    hostname: Hostname,
    key: Key,
    value: u64,
}

#[derive(Debug, Clone)]
pub struct RemoteFloatMetric {
    hostname: Hostname,
    key: Key,
    value: f64,
}

#[derive(Debug, Clone)]
pub struct RemoteStateMetric {
    hostname: Hostname,
    key: Key,
    timestamp: Timestamp,
    value: String,
}

impl RemoteMetric {
    fn from_local(m: Metric, hostname: Hostname) -> RemoteMetric {
        use self::Metric as M;
        match m {
            M::Integer(IntegerMetric { key, value })
            => RM::Integer(RemoteIntegerMetric { key, value, hostname }),
            M::Counter(CounterMetric { key, value })
            => RM::Counter(RemoteCounterMetric { key, value, hostname }),
            M::Float(FloatMetric { key, value })
            => RM::Float(RemoteFloatMetric { key, value, hostname }),
            M::State(StateMetric { key, timestamp, value })
            => RM::State(RemoteStateMetric { key, timestamp, value, hostname }),
        }
    }
}

graphql_object!(IntegerMetric: () |&self| {
    interfaces: [&Metric],
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
    interfaces: [&Metric],
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
    interfaces: [&Metric],
    field key() -> &Key { &self.key }
    field value() -> f64 { self.value }
});

graphql_object!(StateMetric: () |&self| {
    interfaces: [&Metric],
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

graphql_object!(RemoteIntegerMetric: () |&self| {
    interfaces: [&Metric, &RemoteMetric],
    field hostname() -> &Hostname { &self.hostname }
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

graphql_object!(RemoteCounterMetric: () |&self| {
    interfaces: [&Metric, &RemoteMetric],
    field hostname() -> &Hostname { &self.hostname }
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

graphql_object!(RemoteFloatMetric: () |&self| {
    interfaces: [&Metric, &RemoteMetric],
    field hostname() -> &Hostname { &self.hostname }
    field key() -> &Key { &self.key }
    field value() -> f64 { self.value }
});

graphql_object!(RemoteStateMetric: () |&self| {
    interfaces: [&Metric, &RemoteMetric],
    field hostname() -> &Hostname { &self.hostname }
    field key() -> &Key { &self.key }
    field timestamp() -> &Timestamp { &self.timestamp }
    field value() -> &String { &self.value }
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

graphql_interface!(RemoteMetric: () |&self| {
    field key() -> &Key {
        use self::RemoteMetric::*;
        match *self {
            | Integer(RemoteIntegerMetric { ref key, .. })
            | Counter(RemoteCounterMetric { ref key, .. })
            | Float(RemoteFloatMetric { ref key, .. })
            | State(RemoteStateMetric { ref key, .. })
            => key,
        }
    }

    field hostname() -> &Hostname {
        use self::RemoteMetric::*;
        match *self {
            | Integer(RemoteIntegerMetric { ref hostname, .. })
            | Counter(RemoteCounterMetric { ref hostname, .. })
            | Float(RemoteFloatMetric { ref hostname, .. })
            | State(RemoteStateMetric { ref hostname, .. })
            => hostname,
        }
    }

    instance_resolvers: |_| {
        &RemoteIntegerMetric => maybe!(m if let RM::Integer(ref m) = *self),
        &RemoteCounterMetric => maybe!(m if let RM::Counter(ref m) = *self),
        &RemoteFloatMetric => maybe!(m if let RM::Float(ref m) = *self),
        &RemoteStateMetric => maybe!(m if let RM::State(ref m) = *self),
    }
});

pub fn query<'x>(ctx: &ContextRef<'x>, filter: Filter)
    -> Vec<Metric>
{
    let  Filter { exact_key } = filter;
    if let Some(key) = exact_key {
        get_metrics(ctx.stats, &key.into())
    } else {
        Vec::new()
    }
}

pub fn get_metrics(stats: &Stats, filter: &tracking::Filter)
    -> Vec<Metric>
{
    let ref key = filter.exact_key;

    let metric =
        stats.history.tip.values.get(&key)
            .map(|(_, met)| met.clone())
        .or_else(|| stats.history.fine.values.get(&key)
                    .and_then(|hist| {
                        hist.tip_or_none(stats.history.fine.age)
                    }))
        .map(|met| {
            match met {
                TipValue::Counter(v) => Metric::Counter(CounterMetric {
                    key: Key(key.clone()),
                    value: v,
                }),
                TipValue::Integer(v) => Metric::Integer(IntegerMetric {
                    key: Key(key.clone()),
                    value: v,
                }),
                TipValue::Float(v) => Metric::Float(FloatMetric {
                    key: Key(key.clone()),
                    value: v,
                }),
                TipValue::State(v) => Metric::State(StateMetric {
                    key: Key(key.clone()),
                    timestamp: Timestamp::from_ms(v.0),
                    value: v.1,
                }),
            }
        });
    // Option<Metric> to 1 element or 0 element vector
    metric.into_iter().collect()
}

pub fn query_remote<'x>(ctx: &ContextRef<'x>, filter: Filter)
    -> Vec<RemoteMetric>
{
    let  Filter { exact_key } = filter;
    if let Some(key) = exact_key {
        let filter = key.into();
        let mut response = ctx.remote.query_remote(&filter);

        // Always add metrics of itself
        response.extend(
            get_metrics(ctx.stats, &filter)
            .into_iter()
            .map(|m| RemoteMetric::from_local(m, ctx.hostname.clone())));

        return response;
    } else {
        Vec::new()
    }
}

impl From<Vec<Pair>> for tracking::Filter {
    fn from(v: Vec<Pair>) -> tracking::Filter {
        let mut pairs = BTreeMap::new();
        for item in &v {
            pairs.insert(&item.key[..], &item.value[..]);
        }
        let key = history::Key::unsafe_from_iter(pairs.into_iter());
        return tracking::Filter {
            exact_key: key,
        }
    }
}

impl Into<tracking::Filter> for InternalFilter {
    fn into(self) -> tracking::Filter {
        let mut pairs = BTreeMap::new();
        for item in &self.exact_key {
            pairs.insert(&item.key[..], &item.value[..]);
        }
        let key = history::Key::unsafe_from_iter(pairs.into_iter());
        return tracking::Filter {
            exact_key: key,
        }
    }
}
