use std::thread;
use std::sync::{Arc, RwLock};

use rotor::{Loop, Config, Machine, EventSet};
use rotor_carbon::{Fsm as Carbon, connect_ip};
use rotor_tools::timer::{IntervalFunc, interval_func};
use rotor_tools::loop_ext::{LoopExt, LoopInstanceExt};
use rotor_tools::{Duration};

use configs::Configs;
use stats::Stats;
use carbon;

struct Context {
    stats: Arc<RwLock<Stats>>,
}

rotor_compose!(enum Fsm/Seed<Context> {
    Carbon(Carbon<Context>),
    CarbonTimer(IntervalFunc<Context>),
});


// All new async things should be in rotor main loop
pub fn start(configs: &Configs, stats: Arc<RwLock<Stats>>) {
    let loop_creator = Loop::new(&Config::new()).unwrap();
    let mut loop_inst = loop_creator.instantiate(Context {
        stats: stats,
    });

    for cfg in &configs.carbon {
        let sink = loop_inst.add_and_fetch(Fsm::Carbon, |scope| {
            info!("Connecting to carbon at {}:{} (interval {})",
                cfg.host, cfg.port, cfg.interval);
            connect_ip(
                format!("{}:{}", cfg.host, cfg.port).parse().unwrap(),
                scope)
        }).unwrap();
        let cfg = cfg.clone();
        loop_inst.add_machine_with(|scope| {
            Ok(Fsm::CarbonTimer(interval_func(scope,
                Duration::seconds(cfg.interval as i64), move |ctx| {
                    debug!("Sending data to carbon {}:{}",
                        cfg.host, cfg.port);
                    carbon::send(&mut sink.sender(), &cfg,
                                 &*ctx.stats.read()
                                   .expect("Can't lock stats"));
                })))
        }).unwrap();
    }
    thread::spawn(move || {
        loop_inst.run().unwrap();
    });
}
