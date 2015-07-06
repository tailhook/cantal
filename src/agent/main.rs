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
#[macro_use] extern crate mime;
extern crate httparse;
extern crate bytes;
extern crate hyper;

use std::thread;
use std::fs::File;
use std::path::PathBuf;
use std::sync::{RwLock,Arc,mpsc};

use rustc_serialize::Decodable;
use cbor::{Decoder};
use argparse::{ArgumentParser, Store, ParseOption};


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
mod rules;
mod p2p;
mod http;


fn main() {
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
    // TODO(tailhook) just borrow, when scoped work again
    let stats = Arc::new(RwLock::new(stats::Stats::new()));
    let stats_copy1 = stats.clone();
    let stats_copy2 = stats.clone();
    let stats_copy3 = stats.clone();
    let cell = Arc::new(util::Cell::new());
    let cell_copy1 = cell.clone();
    let cell_copy2 = cell.clone();
    let host_copy1 = host.clone();

    let _storage = storage_dir.as_ref().map(|path| {
        let result = File::open(&path.join("current.cbor"))
            .map_err(|e| error!("Error reading old data: {}. Ignoring...", e))
            .and_then(|f| Decoder::from_reader(f).decode().next()
                .ok_or_else(|| error!(
                    "Error parsing old data: No data. Ignoring..."))
                .and_then(|r| r.map_err(|e| error!(
                    "Error parsing old data {:?}. Ignoring...", e)
                )));
        if let Ok(history) = result {
            stats.write().unwrap().history = history;
        }
        let path = path.clone();
        thread::spawn(move || {
            storage::storage_loop(&*cell_copy1, &path, &*stats_copy1)
        })
    });

    let _scan = thread::spawn(move || {
        scanner::scan_loop(&*stats_copy2, storage_dir.map(|_| &*cell_copy2))
    });

    let (tx, rx) = mpsc::channel();
    let _p2p = thread::spawn(move || {
        p2p::p2p_loop(&*stats_copy3, &host_copy1, port, tx)
    });
    let chan = rx.recv().unwrap();

    match server::run_server(&*stats, &host, port, chan) {
        Ok(()) => {}
        Err(x) => {
            error!("Error running server: {}", x);
            std::process::exit(1);
        }
    }
}
