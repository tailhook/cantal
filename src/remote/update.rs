use history::{History, Backlog, Value, compare_timestamps};
use query::Dataset;


/// Checks and inserts timestamps which are not yet present in history
/// Returns how many *valid* timestamps are there (usually all, but...)
fn insert_timestamps(bl: &mut Backlog, timestamps: &Vec<u64>) -> usize {
    if bl.timestamps.len() == 0 {
        bl.timestamps = timestamps.iter().map(|&x| (x, 0)).collect();
        bl.age += bl.timestamps.len() as u64;
        return bl.timestamps.len();
    } else {
        let (diff, valid) = compare_timestamps(timestamps, &bl.timestamps);
        bl.age += diff;
        for &ts in timestamps[..diff as usize].iter().rev() {
            bl.timestamps.push_front((ts, 0));
        }
        return valid;
    }
}

pub fn update_history(hist: &mut History, datasets: Vec<Dataset>) {
    use query::Dataset::*;
    for dset in datasets.into_iter() {
        match dset {
            SingleSeries(_, _, _) => {
                error!("Single series is not expected here");
            }
            MultiSeries(vec) => {
                for (key, chunk, ts) in vec.into_iter() {
                    if ts.len() <= 0 {
                        debug!("Got empty timestamps {:?} {:?}", key, chunk);
                        continue;
                    }
                    let valid = insert_timestamps(&mut hist.fine, &ts);
                    let mut iter = chunk.iter().enumerate().rev();
                    // Find first valid datapoint
                    if let Some((foff, mut fval)) = iter.by_ref()
                        .find(|&(off, ref val)| off < valid && val.is_some())
                    {
                        let foff = foff as u64;
                        let age = hist.fine.age;
                        let vhist = hist.fine.values.entry(key)
                            .or_insert_with(|| {
                                Value::new(&fval.take().unwrap(), age - foff)
                            });
                        if fval.is_some() {
                            if !vhist.push(fval.as_ref().unwrap(), age - foff)
                            {
                                *vhist = Value::new(&fval.as_ref().unwrap(),
                                    age - foff);
                            }
                        };
                        for (off, val_opt) in iter {
                            let off = off as u64;
                            if let Some(val) = val_opt {
                                vhist.push(&val, age - off);
                            }
                        }
                    }
                }
            }
            SingleTip(_, _, _) => {
                error!("Single series is not expected here");
            }
            MultiTip(_vec) => {
                error!("Multi tips are not supported yet");
                /*
                for (_key, _value, _ts) in vec.into_iter() {
                }
                */
            }
            Chart(_) => {
                error!("Chart is not expected here");
            }
            Empty => {}
            Incompatible(e) => {
                error!("Incompatible data pulled from websocket {:?}", e);
            }
        }
    }
}
