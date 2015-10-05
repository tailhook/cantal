use std::mem::{replace, size_of_val};
use std::collections::HashMap;

use Key;
use values::Value as TipValue;
use serialize::json::{Json, ToJson};


#[derive(Debug)]
pub struct Tip {
    // Made pub for serializer, may be fix it?
    pub latest_timestamp: (u64, u32),
    pub values: HashMap<Key, (u64, TipValue)>,
}

// Named fields are ok since we don't store lots of History objects
probor_struct_encoder_decoder!(Tip {
    latest_timestamp => (),
    values => (),
});

impl Tip {
    pub fn new() -> Tip {
        Tip {
            latest_timestamp: (0, 0),
            values: HashMap::new(),
        }
    }
    pub fn info(&self) -> Json {
        let mut key_bytes = 0;
        let mut value_bytes = 0;
        for (k, v) in self.values.iter() {
            key_bytes += k.size();
            value_bytes += size_of_val(v) + v.1.additional_bytes();
        }
        return Json::Object(vec![
            ("values".to_string(), self.values.len().to_json()),
            ("key_bytes".to_string(), key_bytes.to_json()),
            ("value_bytes".to_string(), value_bytes.to_json()),
            ].into_iter().collect());
    }
    pub fn push<'x, I>(&mut self, timestamp: (u64, u32), iter: I)
        where I: Iterator<Item=(&'x Key, &'x TipValue)>
    {
        self.latest_timestamp = timestamp;
        for (k, v) in iter {
            // fast path should be get_mut
            if let Some(ptr) = self.values.get_mut(k) {
                // Only if no key or conflicting type clone the key
                *ptr = (timestamp.0, v.clone());
                continue;
            }
            self.values.insert(k.clone(), (timestamp.0, v.clone()));
        }
    }
    pub fn truncate_by_time(&mut self, timestamp: u64) {
        self.values = replace(&mut self.values, HashMap::new()).into_iter()
            .filter(|&(_, (ts, _))| ts >= timestamp)
            .collect();
    }
}
