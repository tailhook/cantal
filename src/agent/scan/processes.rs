use std::str::FromStr;
use std::io::BufferedReader;
use std::io::EndOfFile;
use std::str::from_utf8;
use std::hash::{Hash};
use std::io::fs::{File, readdir};
use std::collections::{HashMap, BTreeMap};
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::hash_map::Hasher;

use cantal::Metadata;

type Pid = u32;

pub struct ReadCache {
    metadata: HashMap<Path, Metadata>,
}

#[derive(Encodable)]
struct MinimalProcess {
    pid: Pid,
    ppid: Pid,
    name: String,
    state: char,
    vsize: u64,
    rss: u64,
    num_threads: u32,
    start_time: u64,
    user_time: u32,
    system_time: u32,
    child_user_time: u32,
    child_system_time: u32,
    cmdline: String,
    cantal_path: Option<Path>,
}

/*
struct Group {
    pids: Vec<u32>,
    path: Path,
}
*/

#[derive(Encodable, Default)]
pub struct Processes {
    all: Vec<MinimalProcess>,
}

fn get_env_var(pid: u32) -> Option<Path> {
    File::open(&Path::new(format!("/proc/{}/environ", pid)))
    .map(|f| BufferedReader::new(f))
    .and_then(|mut f| {
        loop {
            let line = match f.read_until(0) {
                Ok(line) => line,
                Err(ref e) if e.kind == EndOfFile => {
                    return Ok(None);
                }
                Err(e) => return Err(e),
            };

            if line.starts_with(b"CANTAL_PATH=") {
                return Ok(Some(Path::new(
                    &line["CANTAL_PATH=".len()..line.len()-1])));
            }
        }
    })
    .map_err(|e| debug!("Can't read environ file: {}", e))
    .ok()
    .and_then(|opt| opt)
}

fn read_process(cache: &mut ReadCache, pid: Pid)
    -> Result<MinimalProcess, ()>
{
    let cantal_path = get_env_var(pid);

    let path = Path::new(format!("/proc/{}/cmdline", pid));
    let mut cmdlinebuf = Vec::with_capacity(4096);
    try!(File::open(&path)
        .and_then(|mut f| f.push(4096, &mut cmdlinebuf))
        .map_err(|e| {
            if e.kind != EndOfFile {
                debug!("Can't read cmdline file: {}", e);
            }
        }));
    let cmdline = String::from_utf8_lossy(cmdlinebuf.as_slice());

    let path = Path::new(format!("/proc/{}/stat", pid));
    let mut buf = Vec::with_capacity(4096);
    try!(File::open(&path)
        .and_then(|mut f| f.push(4096, &mut buf))
        .map_err(|e| debug!("Can't read stat file: {}", e)));
    if buf.len() >= 4096 {
        error!("Stat line too long");
        return Err(());
    }
    let name_start = try!(buf.position_elem(&b'(').ok_or(()));
    // Since there might be brackets in the name itself we should use last
    // closing paren
    let name_end = try!(buf.rposition_elem(&b')').ok_or(()));
    let name = try!(from_utf8(&buf[name_start+1..name_end])
        .map_err(|e| debug!("Can't decode stat file: {}", e)))
        .to_string();

    let mut words = try!(from_utf8(&buf[name_end+1..])
        .map_err(|e| debug!("Can't decode stat file: {}", e)))
        .words();

    return Ok(MinimalProcess {
        pid: pid,
        name: name,
        state: words.nth(0).unwrap().char_at(0),
        ppid: words.nth(0).and_then(FromStr::from_str).unwrap(),
        user_time: words.nth(9).and_then(FromStr::from_str).unwrap(),
        system_time: words.nth(0).and_then(FromStr::from_str).unwrap(),
        child_user_time: words.nth(0).and_then(FromStr::from_str).unwrap(),
        child_system_time: words.nth(0).and_then(FromStr::from_str).unwrap(),
        num_threads: words.nth(2).and_then(FromStr::from_str).unwrap(),
        start_time: words.nth(1).and_then(FromStr::from_str).unwrap(),
        vsize: words.nth(0).and_then(FromStr::from_str).unwrap(),
        rss: words.nth(0).and_then(FromStr::from_str).unwrap(),
        cmdline: cmdline.to_string(),
        cantal_path: cantal_path,
    });
    /*
        cantal_path.map(|path| {
            let meta = match Metadata::read(&path.with_extension("meta")) {
                Ok(meta) => meta,
                Err(e) => {
                    warn!("Error parsing metadata {:?}: {}", path, e);
                    return;
                }
            };
            let data = match meta.read_data(&path.with_extension("values")) {
                Ok(data) => data,
                Err(e) => {
                    warn!("Error parsing data {:?}: {}", path, e);
                    return;
                }
            };
            for &(ref descr, ref item) in data.iter() {
                println!("{} {} {:?}", pid, descr.textname, item);
            }
        });
    */
}

fn tree_collect<K: Hash<Hasher> + Eq, V, I: Iterator<Item=(K, V)>>(mut iter: I)
    -> HashMap<K, Vec<V>>
{
    let mut result = HashMap::new();
    for (k, v) in iter {
        if let Some(vec) = result.get_mut(&k) {
            let mut val: &mut Vec<V> = vec;
            val.push(v);
            continue;
        }
        result.insert(k, vec!(v));
    }
    return result;
}

pub fn read(cache: &mut ReadCache) -> Processes {
    let file_list = readdir(&Path::new("/proc"))
        .map_err(|e| error!("Error listing /proc: {}", e))
        .unwrap_or(Vec::new());
    let processes = file_list.into_iter()
        .map(|x| x.filename_str().and_then(FromStr::from_str))
        .filter(|x| x.is_some())
        .map(|x| read_process(cache, x.unwrap()))
        .filter(|x| x.is_ok())
        .map(|x| x.unwrap())
        .collect::<Vec<MinimalProcess>>();

    {
        let children: HashMap<u32, Vec<&MinimalProcess>>;
        children = tree_collect(processes.iter().map(|p| (p.ppid, p)));
    }
    return Processes {
        all: processes,
    };
}

impl ReadCache {
    pub fn new() -> ReadCache {
        ReadCache {
            metadata: HashMap::new(),
        }
    }
}
