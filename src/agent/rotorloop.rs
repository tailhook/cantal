use std::thread;
use std::sync::mpsc::channel;

use rotor::{Loop, Config};
use rotor_carbon::{Sink, connect_ip};


struct Context;

#[derive(Clone)] // currently required to insert into dependencies
pub struct Rotor {
    pub carbon: Sink,
}


// All new async things should be in rotor main loop
pub fn start() -> Rotor {
    let (tx, rx) = channel();
    // We create a loop in the thread. It's simpler to use for demo.
    // But it's perfectly okay to add rotor-carbon thing to your normal
    // event loop
    thread::spawn(move || {
        let mut loop_creator = Loop::new(&Config::new()).unwrap();
        loop_creator.add_machine_with(|scope| {
            let (fsm, sink) = connect_ip(
                &format!("{}:{}", "127.0.0.1", 2003).parse().unwrap(),
                scope).unwrap();
            tx.send(sink).unwrap();
            Ok(fsm)
        }).unwrap();
        loop_creator.run(Context).unwrap();
    });
    Rotor {
        carbon: rx.recv().unwrap()
    }
}
