use history::{History, Value, Chunk, Backlog, TimeStamp};
use values::Value as TipValue;

use {Rule, Source, Dataset, Extract, Function, TimeSlice};

pub fn query_history(rule: &Rule, history: &History) -> Dataset {
    let dset = match rule.series.source {
        Source::Tip => {
            let mut result = Vec::new();
            // TODO(tailhook) do not duplicate keys and values
            for (key, &(ts, ref value)) in history.tip.values.iter() {
                if rule.series.condition.matches(key) {
                    result.push((key.clone(), value.clone(), (ts, ts)));
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
                    .map(|(v, tslc)| result.push((key.clone(), v, tslc)));
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
                    .map(|(v, t)| result.push((key.clone(), v, t)));
                    // TODO(tailhook) if extract_multi returns None what we
                    //                should do?
                }
            }
            Dataset::MultiSeries(result)
        }
    };
    rule.functions.iter().fold(dset, Function::exec)
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
    -> Option<(TipValue, TimeSlice)>
{
    use Extract::*;
    use history::Value as B;
    use values::Value as V;
    match extract {
        &Tip => Some({
            match value {
                &B::Counter(ref x) => {
                    let ts = bl.timestamps[(bl.age - x.age()) as usize].0;
                    (V::Counter(x.tip()), (ts, ts))
                },
                &B::Integer(ref x) => {
                    let ts = bl.timestamps[(bl.age - x.age()) as usize].0;
                    (V::Integer(x.tip()), (ts, ts))
                }
                &B::Float(ref x) => {
                    let ts = bl.timestamps[(bl.age - x.age()) as usize].0;
                    (V::Float(x.tip()), (ts, ts))
                }
            }
        }),
        &DiffToAtMost(n) => {
            match value {
                &B::Counter(ref hist) => {
                    hist.history(bl.age).enumerate().skip(1).take(n)
                     .filter_map(|(idx, x)| x.map(|y| (idx, y))).last()
                     .map(|(idx, x)| {
                        let cur = (bl.age - hist.age()) as usize;
                        assert!(idx >= cur);
                        (V::Counter(hist.tip().saturating_sub(x)),
                            (bl.timestamps[cur].0, bl.timestamps[idx].0))
                    })
                }
                &B::Integer(ref hist) => {
                    hist.history(bl.age).enumerate().skip(1).take(n)
                     .filter_map(|(idx, x)| x.map(|y| (idx, y))).last()
                     .map(|(idx, x)| {
                        let cur = (bl.age - hist.age()) as usize;
                        assert!(idx >= cur);
                        (V::Integer(hist.tip().saturating_sub(x)),
                            (bl.timestamps[cur].0, bl.timestamps[idx].0))
                    })
                }
                &B::Float(ref hist) => {
                    hist.history(bl.age).enumerate().skip(1).take(n)
                     .filter_map(|(idx, x)| x.map(|y| (idx, y))).last()
                     .map(|(idx, x)| {
                        let cur = (bl.age - hist.age()) as usize;
                        assert!(idx >= cur);
                        (V::Float(hist.tip() - x),
                            (bl.timestamps[cur].0, bl.timestamps[idx].0))
                    })
                }
            }
        },
        &HistoryByNum(_) => None,
        &HistoryByTime(_) => None,
    }
}

pub fn extract_multi(value: &Value, bl: &Backlog, extract: &Extract)
    -> Option<(Chunk, Vec<TimeStamp>)>
{
    use Extract::*;
    use history::Value as B;
    use history::Chunk as C;
    match extract {
        // Tip skipped in query_history but may be needed in future
        &Tip => None,
        &DiffToAtMost(_) => None,
        &HistoryByNum(n) => Some({
            let timestamps = bl.timestamps.iter()
                .take(n).map(|&(x, _)| x).collect();
            let values = match value {
                &B::Counter(ref x)
                => C::Counter(x.history(bl.age).take(n).collect()),
                &B::Integer(ref x)
                => C::Integer(x.history(bl.age).take(n).collect()),
                &B::Float(ref x)
                => C::Float(x.history(bl.age).take(n).collect()),
            };
            (values, timestamps)
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
            let timestamps = bl.timestamps.iter()
                .take(num).map(|&(x, _)| x).collect();
            let values = match value {
                &B::Counter(ref x)
                => C::Counter(x.history(bl.age).take(num).collect()),
                &B::Integer(ref x)
                => C::Integer(x.history(bl.age).take(num).collect()),
                &B::Float(ref x)
                => C::Float(x.history(bl.age).take(num).collect()),
            };
            (values, timestamps)
        }),
    }
}
