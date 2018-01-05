extern crate anymap;
extern crate argparse;
extern crate byteorder;
extern crate cbor;
extern crate fern;
extern crate failure;
extern crate futures;
extern crate futures_cpupool;
extern crate hex;
extern crate httparse;
extern crate http_file_headers;
extern crate humantime;
extern crate hyper;
extern crate libc;
extern crate libcantal;
extern crate nix;
extern crate ns_env_config;
extern crate num;
extern crate quire;
extern crate rand;
extern crate regex;
extern crate rustc_serialize;
extern crate scan_dir;
extern crate self_meter_http;
extern crate serde_cbor;
extern crate time;
extern crate tk_carbon;
extern crate tk_bufstream;
extern crate tk_easyloop;
extern crate tk_http;
extern crate tk_listen;
extern crate tokio_core;
extern crate tokio_io;
extern crate unicase;
extern crate void;
extern crate websocket;
extern crate serde;
extern crate serde_json;
extern crate slab;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate log;
#[macro_use] extern crate matches;
#[macro_use] extern crate probor;
#[macro_use] extern crate quick_error;
#[macro_use] extern crate serde_derive;

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
use std::sync::{RwLock, Arc};
use std::process::exit;

use failure::Error;
use nix::unistd::getpid;
use argparse::{ArgumentParser, Store, ParseOption, StoreOption, Parse, Print};
use argparse::{StoreTrue};
use rustc_serialize::json::Json;
use tk_easyloop::{handle};

use deps::{Dependencies, LockedDeps};

mod carbon;
mod configs;
mod deps;
mod frontend;
mod gossip;
mod http;
mod id;
mod info;
mod remote;
mod scan;
mod scanner;
mod stats;
mod storage;
mod time_util;
mod watchdog;


fn main() {
    match run() {
        Ok(()) => {}
        Err(e) => {
            error!("{}", e);
            exit(2);
        }
    }
}

fn run() -> Result<(), Error> {

    let mut name = None;
    let mut host = "127.0.0.1".to_string();
    let mut port = 22682u16;
    let mut storage_dir = None::<PathBuf>;
    let mut config_dir = PathBuf::from("/etc/cantal");
    let mut machine_id = None::<id::Id>;
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

    let logger_result = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!("[{}][{}] {} {}",
                time::now().strftime("%Y-%m-%d %H:%M:%S").unwrap(),
                record.level(), record.location().module_path(), message))
        })
        .level(log_level.unwrap_or(log::LogLevel::Warn).to_log_level_filter())
        .chain(std::io::stderr())
        .apply();
    if let Err(e) = logger_result {
        panic!("Failed to initialize global logger: {}", e);
    }

    let address = SocketAddr::new(host.parse()?, port);

    let meter = self_meter_http::Meter::new();
    meter.track_current_thread("main");

    let configs = Arc::new(configs::read(&config_dir));

    let hostname = info::hostname().unwrap();
    let addresses = info::my_addresses(port).unwrap();
    let name = name.unwrap_or(hostname.clone());
    let machine_id = machine_id.clone()
            .unwrap_or_else(|| info::machine_id());

    let stats = Arc::new(RwLock::new(stats::Stats::new(
        getpid(), name.clone(), hostname.clone(), cluster_name.clone(),
        &machine_id,
        addresses.iter().map(|x| x.to_string()).collect())));
    let mut deps = Dependencies::new();
    deps.insert(stats.clone());

    let (gossip, mut gossip_init) = cluster_name.as_ref().map(|cluster| {
        gossip::Config::new()
        .bind(address)
        .cluster_name(&cluster)
        .machine_id(&machine_id)
        .addresses(&addresses)
        .hostname(&hostname)
        .name(&name)
        .done()
    }).map(|x| gossip::init(&x))
      .map(|(g, i)| (g, Some(i)))
      .unwrap_or_else(|| (gossip::noop(), None));
    deps.insert(gossip.clone());

    let storage = Arc::new(storage::Storage::new());
    deps.insert(storage.clone());

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
            let _watchdog = watchdog::ExitOnReturn(81);
            mymeter.track_current_thread("storage");
            storage::storage_loop(mydeps, &path);
        })
    });
    if let Some(ref path) = storage_dir {
        File::open(&path.join("peers.json"))
        .map_err(|e| error!("Error reading peers: {}. Ignoring...", e))
        .and_then(|mut x| Json::from_reader(&mut x)
        .map_err(|e| error!("Error reading peers: {}. Ignoring...", e)))
        .map(|x| x.find("ip_addresses").and_then(|x| x.as_array())
            .map(|lst| {
                for item in lst {
                    item.as_string()
                    .and_then(|x| SocketAddr::from_str(x).ok())
                    .map(|x| gossip.add_host(x));
                }
            }))
        .ok();
    }

    let mydeps = deps.clone();
    let mymeter = meter.clone();
    let _scan = thread::spawn(move || {
        let _watchdog = watchdog::ExitOnReturn(82);
        mymeter.track_current_thread("scan");
        scanner::scan_loop(mydeps, scan_interval, *backlog_time);
    });

    tk_easyloop::run_forever(|| -> Result<(), Error> {
        let ns = ns_env_config::init(&handle())?;

        meter.spawn_scanner(&handle());

        if let Some(gossip) = gossip_init.take() {
            gossip.spawn(&storage)?;
        }

        carbon::spawn_sinks(&ns, &configs, &stats)?;
        http::spawn_listener(&ns, &host, port, bind_localhost,
            &meter, &stats, &gossip)?;

        Ok(())
    })?;


    Ok(())
}
