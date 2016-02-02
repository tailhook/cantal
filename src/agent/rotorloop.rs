use std::thread;
use std::sync::mpsc::channel;

use configs::Configs;
use rotor::{Loop, Config};
use rotor_carbon::{Sink, connect_ip};


struct Context;

#[derive(Clone)] // currently required to insert into dependencies
pub struct Rotor {
    pub carbon: Vec<(Sink, ())>,
}


// All new async things should be in rotor main loop
pub fn start(configs: &Configs) -> Rotor {
    let (tx, rx) = channel();

    let configs = configs.clone(); // TODO(tailhook) optimize this
    thread::spawn(move || {

        let mut loop_creator = Loop::new(&Config::new()).unwrap();

        let mut carbon = vec!();
        for cfg in configs.carbon {
            loop_creator.add_machine_with(|scope| {
                info!("Connecting to carbon at {}:{}", cfg.host, cfg.port);
                let (fsm, sink) = connect_ip(
                    &format!("{}:{}", cfg.host, cfg.port).parse().unwrap(),
                    scope).unwrap();
                carbon.push((sink, ()));
                Ok(fsm)
            }).unwrap();
        }
        tx.send(Rotor {
            carbon: carbon,
        }).unwrap();

        loop_creator.run(Context).unwrap();

    });
    rx.recv().unwrap()
}
