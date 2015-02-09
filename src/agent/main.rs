extern crate libc;
#[macro_use] extern crate log;

extern crate argparse;

use std::os;
use argparse::{ArgumentParser, Store};

mod aio;
mod server;

fn main() {
    let mut host = "0.0.0.0".to_string();
    let mut port = 22682u16;
    {
        let mut ap = ArgumentParser::new();
        ap.refer(&mut port)
            .add_option(&["-p", "--port"], Box::new(Store::<u16>),
                "Port for http interface");
        ap.refer(&mut host)
            .add_option(&["-h", "--host"], Box::new(Store::<String>),
                "Host for http interface. Warning default one is 0.0.0.0");
        match ap.parse_args() {
            Ok(()) => {}
            Err(x) => {
                os::set_exit_status(x);
                return;
            }
        }
    }
    match server::run_server(host, port) {
        Ok(()) => {}
        Err(x) => {
            error!("Error running server: {}", x);
            os::set_exit_status(1);
            return;
        }
    }
}
