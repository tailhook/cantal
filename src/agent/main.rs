extern crate libc;
#[macro_use] extern crate log;
extern crate cbor;
extern crate argparse;
extern crate rustc_serialize;
extern crate regex;
extern crate nix;
extern crate mio;
extern crate time;
extern crate rand;
extern crate num;
#[macro_use] extern crate mime;
#[macro_use] extern crate matches;
#[macro_use] extern crate probor;
extern crate httparse;
extern crate unicase;
extern crate hyper;
extern crate websocket;
extern crate byteorder;
extern crate anymap;
extern crate fern;

extern crate cantal_values as cantal;
extern crate cantal_history as history;
extern crate cantal_query as query;

use std::env;
use std::thread;
use std::io::BufReader;
use std::fs::File;
use std::str::FromStr;
use std::path::PathBuf;
use std::sync::{RwLock,Arc};
use std::process::exit;
use std::error::Error;
use nix::unistd::gethostname;

use argparse::{ArgumentParser, Store, ParseOption, StoreOption};

use deps::{Dependencies, LockedDeps};


mod util;
mod server;
mod stats;
mod staticfiles;
mod scanner;
mod scan;
mod mountpoints;
mod storage;
mod p2p;
mod http;
mod websock;
mod respond;
mod remote;
mod error;
mod deps;
mod ioutil;


fn hostname() -> String {
    let mut buf = [0u8; 256];
    gethostname(&mut buf).unwrap();
    for (idx, &ch) in buf.iter().enumerate() {
        if ch == 0 {
            return String::from_utf8(buf[..idx].to_owned()).unwrap();
        }
    }
    panic!("Bad hostname");
}



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

    let mut name = None;
    let mut host = "127.0.0.1".to_string();
    let mut port = 22682u16;
    let mut storage_dir = None::<PathBuf>;
    let mut log_level = env::var("RUST_LOG").ok()
        .and_then(|x| FromStr::from_str(&x).ok());
    {
        let mut ap = ArgumentParser::new();
        ap.refer(&mut port)
            .add_option(&["-p", "--port"], Store,
                "Port for http interface");
        ap.refer(&mut host)
            .add_option(&["-h", "--host"], Store,
                "Host for http interface (default 127.0.0.1)");
        ap.refer(&mut name)
            .add_option(&["-n", "--node-name"], StoreOption, "
                Node name to announce. It's used for descriptions and URLs all
                communication is doing without resolving names. By default
                `hostname` is used, but you may want to use fully qualified
                domain name or some name that is visible behind proxy.
            ");
        ap.refer(&mut storage_dir)
            .add_option(&["-d", "--storage-dir"], ParseOption,
                "A directory to serialize data to");
        ap.refer(&mut log_level)
            .add_option(&["--log-level"], StoreOption,
                "Log level");
        ap.parse_args_or_exit();
    }

    let logger_config = fern::DispatchConfig {
        format: Box::new(|msg: &str, level: &log::LogLevel,
                location: &log::LogLocation|
        {
            format!("[{}][{}] {} {}",
                time::now().strftime("%Y-%m-%d %H:%M:%S").unwrap(),
                level, location.module_path(), msg)
        }),
        output: vec![fern::OutputConfig::stderr()],
        level: log_level.unwrap_or(log::LogLevel::Warn).to_log_level_filter(),
    };
    if let Err(e) = fern::init_global_logger(logger_config,
        log_level.unwrap_or(log::LogLevel::Warn).to_log_level_filter())
    {
        panic!("Failed to initialize global logger: {}", e);
    }

    let hostname = hostname();
    let name = name.unwrap_or(hostname.clone());

    let mut deps = Dependencies::new();
    deps.insert(Arc::new(RwLock::new(stats::Stats::new())));

    let p2p_init = try!(p2p::p2p_init(&mut deps, &host, port, hostname, name));
    let server_init = try!(server::server_init(&mut deps, &host, port));

    deps.insert(Arc::new(util::Cell::<storage::Buffer>::new()));

    let _storage = storage_dir.as_ref().map(|path| {
        let mydeps = deps.clone();
        let cborcfg = probor::Config {
            max_len_array: 100000,
            max_len_bytes: 0x500000,
            max_len_text: 0x500000,
            max_size_map: 100000,
            max_nesting: 16,
            .. probor::Config::default()
        };
        let result = File::open(&path.join("current.cbor"))
            .map_err(|e| error!("Error reading old data: {}. Ignoring...", e))
            .map(BufReader::new)
            .map(|f| probor::Decoder::new(cborcfg, f))
            .and_then(|mut dec| {
                let v: history::VersionInfo = try!(probor::decode(&mut dec)
                    .map_err(|_| error!("Can't decode version info. \
                        Ignoring...")));
                if v != history::VersionInfo::current() {
                    error!("Old version of history data. Ignoring...");
                    return Err(());
                }
                probor::decode(&mut dec)
                    .map_err(|e| error!(
                        "Error parsing old data: {}. Ignoring...", e))
            });
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
        p2p::p2p_loop(p2p_init, mydeps)
            .map_err(|e| error!("Error in p2p loop: {}", e))
            .ok();
    });

    try!(server::server_loop(server_init, deps));

    Ok(())
}
