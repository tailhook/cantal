extern crate libc;
#[macro_use] extern crate log;
extern crate cbor;
extern crate argparse;
extern crate cantal;
extern crate rustc_serialize;
extern crate env_logger;
extern crate regex;
extern crate nix;
extern crate mio;
extern crate time;
extern crate rand;
extern crate num;
#[macro_use] extern crate mime;
extern crate httparse;
extern crate unicase;
extern crate hyper;
extern crate websocket;
extern crate byteorder;
extern crate anymap;

use std::thread;
use std::fs::File;
use std::path::PathBuf;
use std::sync::{RwLock,Arc};
use std::process::exit;
use std::error::Error;

use rustc_serialize::Decodable;
use cbor::{Decoder};
use argparse::{ArgumentParser, Store, ParseOption};

use deps::{Dependencies, LockedDeps};


mod util;
mod server;
mod stats;
mod staticfiles;
mod scanner;
mod scan;
mod mountpoints;
mod deltabuf;
mod history;
mod storage;
mod rules;
mod p2p;
mod http;
mod websock;
mod respond;
//mod remote;
mod error;
mod ioloop;
mod deps;


fn main() {
    match run() {
        Ok(()) => {}
        Err(e) => {
            error!("{}", e);
            exit(2);
        }
    }
}

fn run() -> Result<(), Box<Error>> {
    env_logger::init().unwrap();

    let mut host = "127.0.0.1".to_string();
    let mut port = 22682u16;
    let mut storage_dir = None::<PathBuf>;
    {
        let mut ap = ArgumentParser::new();
        ap.refer(&mut port)
            .add_option(&["-p", "--port"], Store,
                "Port for http interface");
        ap.refer(&mut host)
            .add_option(&["-h", "--host"], Store,
                "Host for http interface (default 127.0.0.1)");
        ap.refer(&mut storage_dir)
            .add_option(&["-d", "--storage-dir"], ParseOption,
                "A directory to serialize data to");
        ap.parse_args_or_exit();
    }

    let mut deps = Dependencies::new();
    deps.insert(Arc::new(RwLock::new(stats::Stats::new())));

    let p2p_init = try!(p2p::p2p_init(&mut deps, &host, port));
    let server_init = try!(server::server_init(&mut deps, &host, port));

    deps.insert(Arc::new(util::Cell::<storage::Buffer>::new()));

    let _storage = storage_dir.as_ref().map(|path| {
        let mut mydeps = deps.clone();
        let result = File::open(&path.join("current.cbor"))
            .map_err(|e| error!("Error reading old data: {}. Ignoring...", e))
            .and_then(|f| Decoder::from_reader(f).decode().next()
                .ok_or_else(|| error!(
                    "Error parsing old data: No data. Ignoring..."))
                .and_then(|r| r.map_err(|e| error!(
                    "Error parsing old data {:?}. Ignoring...", e)
                )));
        if let Ok(history) = result {
            mydeps.write::<stats::Stats>().history = history;
        }
        let path = path.clone();
        thread::spawn(move || {
            storage::storage_loop(mydeps, &path);
        })
    });

    let mydeps = deps.clone();
    let _scan = thread::spawn(move || {
        scanner::scan_loop(mydeps);
    });

    let mydeps = deps.clone();
    let _p2p = thread::spawn(move || {
        p2p::p2p_loop(p2p_init, mydeps);
    });

    try!(server::server_loop(server_init, deps));

    Ok(())
}
