use std::hash::{Hash};
use std::collections::HashMap;


pub fn tree_collect<K: Hash + Eq, V, I: Iterator<Item=(K, V)>>(mut iter: I)
    -> HashMap<K, Vec<V>>
{
    let mut result = HashMap::new();
    for (k, v) in iter {
        if let Some(vec) = result.get_mut(&k) {
            let mut val: &mut Vec<V> = vec;
            val.push(v);
            continue;
        }
        result.insert(k, vec!(v));
    }
    return result;
}
