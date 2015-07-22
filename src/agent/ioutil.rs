use std::fmt::Debug;

use mio::{EventLoop, Evented, EventSet, Token, PollOpt, Handler};


/// Trait for event loop that is specific for the app:
///
/// 1. We always PollOpt::level()
/// 2. Reregister/deregister never fails (i.e. fails only when bug in the code)
/// 2. Register never fails (i.e. in case of bug or OOM*)
///
/// (*) It's usually kernel out of memory or limit reached, but there is no
/// realistic situation where kernel has no memory and cantal has, and handling
/// OOM in rust isn't too good anyway (I mean doesn't work in cantal at all)
pub trait Poll {
    fn add<E:?Sized>(&mut self, io: &E, tok: Token, read: bool, write: bool)
        where E: Evented + Debug;
    fn modify<E:?Sized>(&mut self, io: &E, tok: Token, read: bool, write: bool)
        where E: Evented + Debug;
    fn remove<E:?Sized>(&mut self, io: &E)
        where E: Evented + Debug;
}

impl<H:Handler> Poll for EventLoop<H> {
    fn add<E:?Sized>(&mut self, io: &E, tok: Token, read: bool, write: bool)
        where E: Evented + Debug
    {
        assert!(read || write);
        let eset = EventSet::none()
            | (if read { EventSet::readable() } else { EventSet::none() })
            | (if write { EventSet::writable() } else { EventSet::none() });
        self.register_opt(io, tok, eset, PollOpt::level())
            .map_err(|e| {
                error!("Can't register token {:?}, io {:?}: {}", tok, io, e);
                panic!("Can't register token {:?}, io {:?}: {}", tok, io, e);
            }).unwrap();
    }
    fn modify<E:?Sized>(&mut self, io: &E, tok: Token, read: bool, write: bool)
        where E: Evented + Debug
    {
        assert!(read || write);
        let eset = EventSet::none()
            | (if read { EventSet::readable() } else { EventSet::none() })
            | (if write { EventSet::writable() } else { EventSet::none() });
        self.reregister(io, tok, eset, PollOpt::level())
            .map_err(|e| {
                error!("Can't register token {:?}, io {:?}: {}", tok, io, e);
                panic!("Can't register token {:?}, io {:?}: {}", tok, io, e);
            }).unwrap();
    }
    fn remove<E:?Sized>(&mut self, io: &E)
        where E: Evented + Debug
    {
        self.deregister(io)
            .map_err(|e| {
                error!("Can't deregister io {:?}: {}", io, e);
                panic!("Can't deregister io {:?}: {}", io, e);
            }).unwrap();
    }
}
