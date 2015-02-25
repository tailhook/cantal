#![feature(unboxed_closures)]

extern crate libc;
#[macro_use] extern crate log;
extern crate serialize;

extern crate argparse;
extern crate cantal;

use std::env;
use std::thread::Thread;
use std::sync::{RwLock, Arc};
use argparse::{ArgumentParser, Store};


mod aio;
mod server;
mod stats;
mod staticfiles;
mod scanner;
mod scan;


fn main() {
    let mut host = "127.0.0.1".to_string();
    let mut port = 22682u16;
    {
        let mut ap = ArgumentParser::new();
        ap.refer(&mut port)
            .add_option(&["-p", "--port"], Store,
                "Port for http interface");
        ap.refer(&mut host)
            .add_option(&["-h", "--host"], Store,
                "Host for http interface (default 127.0.0.1)");
        match ap.parse_args() {
            Ok(()) => {}
            Err(x) => {
                env::set_exit_status(x);
                return;
            }
        }
    }
    let stats = &Arc::new(RwLock::new(stats::Stats::new()));
    let stats2 = stats.clone();
    Thread::spawn(move || scanner::scan_loop(stats2));
    match server::run_server(&**stats, host, port) {
        Ok(()) => {}
        Err(x) => {
            error!("Error running server: {}", x);
            env::set_exit_status(1);
            return;
        }
    }
}
