use std::cmp::min;

use history::{Value, Backlog, CounterHistory};


pub fn graphite_data(val: &Value, blog: &Backlog, num: usize)
    -> Option<f64>
{
    use history::Value::{Counter, Integer};
    match val {
        &Counter(ref hist) => {
            get_rate_from_counter(hist, blog, num)
            .map(|(value, millis)| {
                (value as f64)*1000.0/(millis as f64)
            })
        }
        &Integer(ref hist)
        if val.age() >= blog.age - min(num as u64, blog.age)
        => {
            Some(hist.tip() as f64)
        }
        _ => None,
    }
}

fn get_rate_from_counter(hist: &CounterHistory, blog: &Backlog, num: usize)
    -> Option<(u64, u64)>
{
    hist.history(blog.age).enumerate().skip(1).take(num)
    .filter_map(|(idx, x)| x.map(|y| (idx, y))).last()
    .map(|(idx, x)| {
        let cur = (blog.age - hist.age()) as usize;
        assert!(idx >= cur);
        (hist.tip().saturating_sub(x),
            (blog.timestamps[cur].0 - blog.timestamps[idx].0))
    })
}

pub fn get_counter_diff(val: &Value, blog: &Backlog, num: usize)
    -> Option<(u64, u64)>
{
    use history::Value::Counter;
    match val {
        &Counter(ref hist) => {
            get_rate_from_counter(hist, blog, num)
        }
        _ => None,
    }
}

