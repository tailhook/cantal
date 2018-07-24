use futures::sync::mpsc::{unbounded, UnboundedSender, UnboundedReceiver};
use tk_easyloop::spawn;


enum Message {
}

pub struct Init {
    rx: UnboundedReceiver<Message>,
}

pub struct Remote {
    tx: UnboundedSender<Message>,
}

pub fn init() -> (Remote, Init) {
    let (tx, rx) = unbounded();
    return (Remote { tx }, Init { rx });
}

impl Init {
    pub fn spawn(self) {
        unimplemented!();
    }
}
