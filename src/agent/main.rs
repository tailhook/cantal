#![feature(unboxed_closures)]

extern crate libc;
#[macro_use] extern crate log;
extern crate serialize;

extern crate argparse;

use std::os;
use std::sync::RwLock;
use argparse::{ArgumentParser, Store};


mod aio;
mod server;
mod stats;
mod staticfiles;


fn main() {
    let mut host = "127.0.0.1".to_string();
    let mut port = 22682u16;
    {
        let mut ap = ArgumentParser::new();
        ap.refer(&mut port)
            .add_option(&["-p", "--port"], Box::new(Store::<u16>),
                "Port for http interface");
        ap.refer(&mut host)
            .add_option(&["-h", "--host"], Box::new(Store::<String>),
                "Host for http interface (default 127.0.0.1)");
        match ap.parse_args() {
            Ok(()) => {}
            Err(x) => {
                os::set_exit_status(x);
                return;
            }
        }
    }
    let stats = RwLock::new(stats::Stats::new());
    match server::run_server(&stats, host, port) {
        Ok(()) => {}
        Err(x) => {
            error!("Error running server: {}", x);
            os::set_exit_status(1);
            return;
        }
    }
}
