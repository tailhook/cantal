use std::sync::{Arc, RwLock};

use cantal::Value;
use frontend::quick_reply::{reply, respond_probor};
use frontend::{Request};
use history::{Key, TimeStamp};
use probor;
use stats::Stats;


struct Response<'x> {
    metrics: Vec<(&'x Key, TimeStamp, Value)>,
}

impl<'x> probor::Encodable for Response<'x> {
    fn encode<W: probor::Output>(&self, e: &mut probor::Encoder<W>)
        -> Result<(), probor::EncodeError>
    {
        probor_enc_struct!(e, self, {
            metrics => (),
        });
        Ok(())
    }
}

pub fn serve<S: 'static>(stats: &Arc<RwLock<Stats>>)
    -> Request<S>
{
    let stats = stats.clone();
    reply(move |e| {
        let stats: &Stats = &*stats.read().expect("stats not poisoned");
        let ref fts = stats.history.fine.timestamps;
        let fage = stats.history.fine.age;
        let vec: Vec<(&Key, TimeStamp, Value)> =
            stats.history.tip.values.iter()
                .map(|(k, &(ts, ref v))| (k, ts, v.clone()))
            .chain(stats.history.fine.values.iter().map(
                |(k, v)| (k, fts[(fage - v.age()) as usize].0, v.tip_value())))
            .collect();
        Box::new(respond_probor(e,
            &Response {
                metrics: vec,
            }))
    })
}
