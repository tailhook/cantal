use std::sync::{RwLock, RwLockWriteGuard, RwLockReadGuard, Arc};
use std::sync::{Mutex, MutexGuard};

use anymap::any::Any;
use anymap::any::CloneAny;
use anymap::Map;

use gossip::Gossip;


pub type Dependencies = Map<CloneAny+Sync+Send>;

pub trait LockedDeps {
    fn write<T:Any+Sync+Send>(&self) -> RwLockWriteGuard<T>;
    fn read<T:Any+Sync+Send>(&self) -> RwLockReadGuard<T>;
    fn lock<T:Any+Send>(&self) -> MutexGuard<T>;
    fn copy<T:Any+Sync+Send>(&self) -> Arc<T>;

    /// This is a hard-coded technical debt for now
    // TODO(tailhook) make Dependencies a structure
    fn gossip(&self) -> &Gossip;
}


impl LockedDeps for Dependencies {
    fn write<T:Any+Sync+Send>(&self) -> RwLockWriteGuard<T> {
        self.get::<Arc<RwLock<T>>>()
        .unwrap().write().unwrap()
    }
    fn read<T:Any+Sync+Send>(&self) -> RwLockReadGuard<T> {
        self.get::<Arc<RwLock<T>>>()
        .unwrap().read().unwrap()
    }
    fn lock<T:Any+Send>(&self) -> MutexGuard<T> {
        self.get::<Arc<Mutex<T>>>().unwrap().lock().unwrap()
    }
    fn copy<T:Any+Sync+Send>(&self) -> Arc<T> {
        self.get::<Arc<T>>().unwrap().clone()
    }
    fn gossip(&self) -> &Gossip {
        self.get::<Gossip>().expect("gossip always exists")
    }
}
