use history::{History, Value, Chunk, Backlog};
use values::Value as TipValue;

use {Rule, Source, Dataset, Extract};

pub fn query_history(rule: &Rule, history: &History) -> Dataset {
    let dset = match rule.series.source {
        Source::Tip => {
            let mut result = Vec::new();
            // TODO(tailhook) do not duplicate keys and values
            for (key, &(_, ref value)) in history.tip.values.iter() {
                if rule.series.condition.matches(key) {
                    result.push((key.clone(), value.clone()));
                }
            }
            Dataset::MultiTip(result)
        }
        Source::Fine if single_value(&rule.extract) => {
            let mut result = Vec::new();
            // TODO(tailhook) do not duplicate keys and values
            for (key, value) in history.fine.values.iter() {
                if rule.series.condition.matches(key) {
                    extract_single(
                        value, &history.fine, &rule.extract)
                    .map(|v| result.push((key.clone(), v)));
                    // TODO(tailhook) if extract_single returns None what we
                    //                should do?
                }
            }
            Dataset::MultiTip(result)
        }
        Source::Fine => {
            let mut result = Vec::new();
            // TODO(tailhook) do not duplicate keys and values
            for (key, value) in history.fine.values.iter() {
                if rule.series.condition.matches(key) {
                    extract_multi(
                        value, &history.fine, &rule.extract)
                    .map(|v| result.push((key.clone(), v)));
                    // TODO(tailhook) if extract_multi returns None what we
                    //                should do?
                }
            }
            Dataset::MultiSeries(result)
        }
    };
    // TODO apply Functions
    dset
}

pub fn single_value(extract: &Extract) -> bool {
    use Extract::*;
    match extract {
        &Tip => true,
        &DiffToAtMost(_) => true,
        &HistoryByNum(_) => false,
        &HistoryByTime(_) => false,
    }
}

pub fn extract_single(value: &Value, bl: &Backlog, extract: &Extract)
    -> Option<TipValue>
{
    use Extract::*;
    use history::Value as B;
    use values::Value as V;
    match extract {
        &Tip => Some({
            match value {
                &B::Counter(ref x) => V::Counter(x.tip()),
                &B::Integer(ref x) => V::Integer(x.tip()),
                &B::Float(ref x) => V::Float(x.tip()),
            }
        }),
        &DiffToAtMost(n) => Some({
            match value {
                &B::Counter(ref x) => V::Counter(x.tip().saturating_sub(
                    x.history(bl.age).take(n)
                     .filter_map(|x| x).last().unwrap())),
                &B::Integer(ref x) => V::Integer(x.tip().saturating_sub(
                    x.history(bl.age).take(n)
                     .filter_map(|x| x).last().unwrap())),
                &B::Float(ref x) => V::Float(x.tip() -
                    x.history(bl.age).take(n)
                     .filter_map(|x| x).last().unwrap()),
            }
        }),
        &HistoryByNum(_) => None,
        &HistoryByTime(_) => None,
    }
}

pub fn extract_multi(value: &Value, bl: &Backlog, extract: &Extract)
    -> Option<Chunk>
{
    use Extract::*;
    use history::Value as B;
    use history::Chunk as C;
    match extract {
        // Tip skipped in query_history but may be needed in future
        &Tip => None,
        &DiffToAtMost(_) => None,
        &HistoryByNum(n) => Some({
            match value {
                &B::Counter(ref x)
                => C::Counter(x.history(bl.age).take(n).collect()),
                &B::Integer(ref x)
                => C::Integer(x.history(bl.age).take(n).collect()),
                &B::Float(ref x)
                => C::Float(x.history(bl.age).take(n).collect()),
            }
        }),
        &HistoryByTime(time_delta) => Some({
            if bl.timestamps.len() < 1 {
                return None;
            }
            let tip = bl.timestamps[0].0;
            let mut num = bl.timestamps.len();
            for (idx, &(ts, _)) in bl.timestamps.iter().enumerate() {
                if tip - ts >= time_delta as u64 {
                    num = idx + 1;
                    break;
                }
            }
            match value {
                &B::Counter(ref x)
                => C::Counter(x.history(bl.age).take(num).collect()),
                &B::Integer(ref x)
                => C::Integer(x.history(bl.age).take(num).collect()),
                &B::Float(ref x)
                => C::Float(x.history(bl.age).take(num).collect()),
            }
        }),
    }
}
