use std::thread;
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;

use rotor::{Loop, Config};
use self_meter::Meter;
use rotor::mio::tcp::TcpStream;
use rotor_carbon::{Fsm as Carbon, connect_ip};
use rotor_tools::timer::{IntervalFunc, interval_func};
use rotor_tools::loop_ext::{LoopInstanceExt};

use configs::Configs;
use stats::Stats;
use carbon;

pub struct Context {
    stats: Arc<RwLock<Stats>>,
}

rotor_compose!(enum Fsm/Seed<Context> {
    Carbon(Carbon<Context, TcpStream>),
    CarbonTimer(IntervalFunc<Context>),
    SelfScanTimer(IntervalFunc<Context>),
});


// All new async things should be in rotor main loop
pub fn start(configs: &Configs, stats: Arc<RwLock<Stats>>,
    meter: Arc<Mutex<Meter>>)
{
    let loop_creator = Loop::new(&Config::new()).unwrap();
    let meter2 = meter.clone();
    let mut loop_inst = loop_creator.instantiate(Context {
        stats: stats,
    });
    loop_inst.add_machine_with(|scope| {
        interval_func(scope,
            Duration::new(1, 0), move |_| {
                meter2.lock().unwrap()
                .scan().map_err(|e| error!("Self-scan error: {}", e)).ok();
            }).wrap(Fsm::SelfScanTimer)
    }).unwrap();

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
            interval_func(scope,
                Duration::new(cfg.interval as u64, 0), move |ctx| {
                        debug!("Sending data to carbon {}:{}",
                            cfg.host, cfg.port);
                        carbon::send(&mut sink.sender(), &cfg,
                                     &*ctx.stats.read()
                                       .expect("Can't lock stats"));
                }).wrap(Fsm::CarbonTimer)
        }).unwrap();
    }
    thread::spawn(move || {
        meter.lock().unwrap().track_current_thread("rotor");
        loop_inst.run().unwrap();
    });
}
