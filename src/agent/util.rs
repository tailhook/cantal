use std::sync::{Mutex, Condvar};
use std::hash::{Hash};
use std::collections::HashMap;

pub struct Cell<T>{
    value: Mutex<Option<T>>,
    cond: Condvar,
}


pub fn tree_collect<K: Hash + Eq, V, I: Iterator<Item=(K, V)>>(iter: I)
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


impl<T:Send + 'static> Cell<T> {
    pub fn new() -> Cell<T> {
        return Cell {
            value: Mutex::new(None),
            cond: Condvar::new(),
        }
    }
    pub fn put(&self, value: T) {
        let mut lock = self.value.lock().unwrap();
        *lock = Some(value);
        self.cond.notify_one();
    }
    pub fn get(&self) -> T {
        loop {
            let lock = self.value.lock().unwrap();
            let mut lock = self.cond.wait(lock).unwrap();
            if let Some(val) = lock.take() {
                return val;
            }
        }
    }
}
