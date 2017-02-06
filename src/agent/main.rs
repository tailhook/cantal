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
#[macro_use] extern crate lazy_static;
extern crate httparse;
extern crate unicase;
extern crate hyper;
extern crate websocket;
extern crate byteorder;
extern crate anymap;
extern crate fern;
extern crate quire;
extern crate scan_dir;
#[macro_use] extern crate rotor;
extern crate rotor_carbon;
extern crate rotor_tools;
extern crate humantime;
extern crate self_meter;
extern crate futures;
extern crate tokio_core;
extern crate tk_easyloop;
extern crate void;
#[macro_use] extern crate quick_error;

extern crate cantal_values as cantal;
extern crate cantal_history as history;
extern crate cantal_query as query;

use std::env;
use std::thread;
use std::io::BufReader;
use std::fs::File;
use std::net::SocketAddr;
use std::str::FromStr;
use std::path::PathBuf;
use std::time::Duration;
use std::sync::{RwLock, Arc, Mutex};
use std::process::exit;
use std::error::Error;

use mio::Sender;
use nix::unistd::getpid;
use argparse::{ArgumentParser, Store, ParseOption, StoreOption, Parse, Print};
use argparse::{StoreTrue};
use rustc_serialize::hex::{ToHex, FromHex};
use rustc_serialize::json::Json;

use deps::{Dependencies, LockedDeps};

pub type HostId = Vec<u8>;

mod util;
mod server;
mod stats;
mod staticfiles;
mod scanner;
mod scan;
mod storage;
mod p2p;
mod gossip;
mod http;
mod websock;
mod respond;
mod remote;
mod error;
mod deps;
mod ioutil;
mod info;
mod rotorloop;
mod carbon;
mod configs;
mod tokioloop;
mod time_util;


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
    let mut config_dir = PathBuf::from("/etc/cantal");
    let mut machine_id = None::<String>;
    let mut cluster_name = None::<String>;
    let mut scan_interval = 2000;
    let mut bind_localhost = false;
    let mut backlog_time = humantime::Duration::from_str("1 hour").unwrap();
    let mut log_level = env::var("RUST_LOG").ok()
        .and_then(|x| FromStr::from_str(&x).ok());
    {
        let mut ap = ArgumentParser::new();
        ap.add_option(&["--version"],
            Print(env!("CARGO_PKG_VERSION").to_string()),
            "Show version and exit");
        ap.refer(&mut port)
            .add_option(&["-p", "--port"], Store,
                "Port for http interface");
        ap.refer(&mut host)
            .add_option(&["-h", "--host"], Store,
                "Host for http interface (default 127.0.0.1).
                 If you change this, it's also a good idea to add
                 --bind-localhost");
        ap.refer(&mut bind_localhost)
            .add_option(&["--bind-localhost"], StoreTrue,
                "Bind localhost together with specified host.
                 This is useful if you want bind cantal to be directly
                 accessible under intranet IP and also on localhost for local
                 tools.");
        ap.refer(&mut name)
            .add_option(&["-n", "--node-name"], StoreOption, "
                Node name to announce. It's used for descriptions and URLs all
                communication is doing without resolving names. By default
                `hostname` is used, but you may want to use fully qualified
                domain name or some name that is visible behind proxy.
            ");
        ap.refer(&mut scan_interval)
            .add_option(&["-i", "--interval"], Store,
            "Scan interval in milliseconds (default 2000 ms).
             Note this is only partially implemented.");
        ap.refer(&mut backlog_time)
            .add_option(&["--keep-history"], Store,
            "Sets amount of history that is stored by cantal in-memory.
             If this value is set to less that 1 hour we also disable hourly
             snapshots (because it makes them useless)");
        ap.refer(&mut cluster_name)
            .add_option(&["-n", "--cluster-name"], StoreOption, "
                A name of the cluster. If cantal receives ping packet with
                mismatching cluster name it discards the packet. If name is
                not specified, cantal will not support discovery.
            ");
        ap.refer(&mut machine_id)
            .add_option(&["--override-machine-id"], StoreOption, "
                Overrides machine id. Do not use in production, put the
                file `/etc/machine-id` instead. This should only be used
                for tests which run multiple nodes in single filesystem
                image");
        ap.refer(&mut storage_dir)
            .add_option(&["-d", "--storage-dir"], ParseOption,
                "A directory to serialize data to");
        ap.refer(&mut config_dir)
            .add_option(&["-c", "--config-dir"], Parse,
                "A directory with configuration files");
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

    let address = SocketAddr::new(host.parse()?, port);

    let meter = Arc::new(Mutex::new(
        self_meter::Meter::new(Duration::new(1, 0))
        .expect("meter created")));
    meter.lock().unwrap().track_current_thread("main");

    let configs = configs::read(&config_dir);

    let hostname = info::hostname().unwrap();
    let addresses = info::my_addresses(port).unwrap();
    let name = name.unwrap_or(hostname.clone());
    let machine_id = machine_id
        .map(|x| x.from_hex().expect("valid machine-id"))
        .unwrap_or_else(info::machine_id);

    let stats = Arc::new(RwLock::new(stats::Stats::new(
        getpid(), name.clone(), hostname.clone(), cluster_name.clone(),
        machine_id.to_hex(),
        addresses.iter().map(|x| x.to_string()).collect())));
    let mut deps = Dependencies::new();
    deps.insert(stats.clone());
    deps.insert(meter.clone());

    let mut gossip = cluster_name.as_ref().map(|cluster| {
        gossip::Config::new()
        .bind(address)
        .cluster_name(&cluster)
        .machine_id(&machine_id)
        .addresses(&addresses)
        .hostname(&hostname)
        .name(&name)
        .done()
    }).map(|x| gossip::init(&x));
    /*
    let p2p_init = try!(p2p::p2p_init(&mut deps, &host, port,
        machine_id, addresses.clone(),
        hostname.clone(), name.clone(), cluster_name.clone()));
    */
    let server_init = try!(
        server::server_init(&mut deps, &host, port, bind_localhost));

    deps.insert(Arc::new(storage::Storage::new()));

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
        let mymeter = meter.clone();
        thread::spawn(move || {
            mymeter.lock().unwrap().track_current_thread("storage");
            storage::storage_loop(mydeps, &path);
        })


    });
    if let Some(ref path) = storage_dir {
        let p2p_chan = deps.get::<Sender<_>>().unwrap();
        File::open(&path.join("peers.json"))
        .map_err(|e| error!("Error reading peers: {}. Ignoring...", e))
        .and_then(|mut x| Json::from_reader(&mut x)
        .map_err(|e| error!("Error reading peers: {}. Ignoring...", e)))
        .map(|x| x.find("ip_addresses").and_then(|x| x.as_array())
            .map(|lst| {
                for item in lst {
                    item.as_string()
                    .and_then(|x| SocketAddr::from_str(x).ok())
                    .and_then(|x| {
                        gossip.as_ref().map(|&(ref g, _)| g.add_host(x));
                        p2p_chan.send(p2p::Command::AddGossipHost(x)).ok()
                    }); // ignore bad hosts
                }
            }))
        .ok();
    }

    let mydeps = deps.clone();
    let mymeter = meter.clone();
    let _scan = thread::spawn(move || {
        mymeter.lock().unwrap().track_current_thread("scan");
        scanner::scan_loop(mydeps, scan_interval, *backlog_time);
    });

    /*
    let mydeps = deps.clone();
    let mymeter = meter.clone();
    let _p2p = thread::spawn(move || {
        mymeter.lock().unwrap().track_current_thread("gossip");
        p2p::p2p_loop(p2p_init, mydeps)
            .map_err(|e| error!("Error in p2p loop: {}", e))
            .ok();
    });
    */

    tokioloop::start(gossip.take().map(|(_, x)| x), &configs, &stats, &meter);

    rotorloop::start(&configs, &stats, &meter);

    try!(server::server_loop(server_init, deps));

    Ok(())
}
