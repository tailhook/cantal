use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::path::Component::Normal;
use std::collections::HashMap;

use scan_dir::ScanDir;

use super::processes::Pid;

pub struct CGroup {
    pub pids: Vec<Pid>,
}

pub type CGroups = HashMap<String, CGroup>;


fn get_name_dir() -> Option<PathBuf> {
    let base = Path::new("/sys/fs/cgroup"); // should customize this?
    let mut buf = String::with_capacity(1024);
    if let Err(_) = File::open("/proc/self/cgroup")
        .and_then(|mut f| f.read_to_string(&mut buf))
        .map_err(|e| debug!("Can't read cgroup file: {}", e))
    {
        return None;
    }
    for line in buf.lines() {
        let mut chunks = line.split(':');
        chunks.next();
        match chunks.next() {
            Some(x) => {
                let mut iter = x.split('=');
                match (iter.next(), iter.next()) {
                    (Some(names), Some(dir)) => {
                        for item in names.split(',') {
                            if item == "name" {
                                return Some(base.join(dir));
                            }
                        }
                    }
                    (Some(names), None) => {
                        for item in names.split(',') {
                            if item == "name" {
                                return Some(base.join(names));
                            }
                        }
                    }
                    _ => {}
                }
            }
            None => {}
        }
    }
    debug!("Couldn't find name cgroup");
    None
}

fn make_name(path: &Path, skip_prefix: usize) -> String {
    let mut buf = String::with_capacity(16);
    for mut cmp in path.components().skip(skip_prefix) {
        if let Normal(name_os) = cmp {
            if let Some(mut name) = name_os.to_str() {
                if name.ends_with(".slice") || name.ends_with(".scope") {
                    name = &name[..name.len()-6];
                } else if name.ends_with(".service") {
                    name = &name[..name.len()-8];
                } else if name == "cgroup.procs" {
                    break;
                }
                if buf.len() > 0 {
                    buf.push('.')
                }
                buf.push_str(name);
            }
        }
    }
    return buf;
}

pub fn read() -> CGroups {
    let mut pro = HashMap::new();
    let mut buf = String::with_capacity(1024);
    // TODO(tailhook) should this be cached?
    if let Some(name_dir) = get_name_dir() {
        let prefix_num = name_dir.components().count();
        ScanDir::dirs().walk(name_dir, |iter| {
            for (entry, name) in iter {
                if name.ends_with(".swap") || name.ends_with(".mount") {
                    // Systemd stuff, not very interesting here
                    continue;
                }
                let mut path = entry.path();
                path.push("cgroup.procs");
                buf.truncate(0);
                if File::open(&path)
                    .and_then(|mut f| f.read_to_string(&mut buf))
                    .map_err(|e| debug!("Error reading cgroup {:?}: {}",
                                       path, e))
                    .is_err()
                {
                    continue;
                }
                let pids: Vec<Pid> = buf.split_whitespace()
                    .filter_map(|x| x.parse().ok())
                    .collect();
                if pids.len() > 0 {
                    let name = make_name(&path, prefix_num);
                    pro.insert(name, CGroup { pids: pids });
                }
            }
        }).map_err(|e| debug!("Error reading directory")).ok();
    }
    return pro;
}
