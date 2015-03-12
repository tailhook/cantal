#![feature(env, rustc_private)]

extern crate libc;
#[macro_use] extern crate log;
extern crate serialize;
extern crate msgpack;

extern crate argparse;
extern crate cantal;

use std::env;
use std::thread;
use std::old_io::fs::File;
use std::sync::RwLock;

use serialize::Decodable;
use msgpack::{Decoder};
use argparse::{ArgumentParser, Store, StoreOption};


mod aio;
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


fn main() {
    let mut host = "127.0.0.1".to_string();
    let mut port = 22682u16;
    let mut storage_dir = None::<Path>;
    {
        let mut ap = ArgumentParser::new();
        ap.refer(&mut port)
            .add_option(&["-p", "--port"], Store,
                "Port for http interface");
        ap.refer(&mut host)
            .add_option(&["-h", "--host"], Store,
                "Host for http interface (default 127.0.0.1)");
        ap.refer(&mut storage_dir)
            .add_option(&["-d", "--storage-dir"], StoreOption,
                "A directory to serialize data to");
        match ap.parse_args() {
            Ok(()) => {}
            Err(x) => {
                env::set_exit_status(x);
                return;
            }
        }
    }
    let stats = &RwLock::new(stats::Stats::new());
    let cell = &util::Cell::new();

    let _storage = storage_dir.as_ref().map(|path| {
        let result = File::open(&path.join("current.msgpack"))
            .and_then(|f| Decodable::decode(&mut Decoder::new(f)))
            .map_err(|e| error!("Error reading old data: {}. Ignoring...", e));
        if let Ok(history) = result {
            stats.write().unwrap().history = history;
        }
        let path = path.clone();
        thread::scoped(move || {
            storage::storage_loop(cell, &path, stats)
        })
    });

    let _scan = thread::scoped(move || {
        scanner::scan_loop(stats, storage_dir.map(|_| cell))
    });

    match server::run_server(stats, host, port) {
        Ok(()) => {}
        Err(x) => {
            error!("Error running server: {}", x);
            env::set_exit_status(1);
            return;
        }
    }
}
