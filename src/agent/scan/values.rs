use std::rc::Rc;
use std::io::{BufReader, BufRead};
use std::fs::{File};
use std::ffi::{OsStr, OsString};
use std::str;
use std::ascii::AsciiExt;
use std::os::unix::prelude::OsStrExt;
use std::path::{Path, PathBuf};
use std::collections::{HashMap};

use cantal::{Metadata, Value, Descriptor};
use rustc_serialize::json::Json;

use super::Tip;
use super::super::util::tree_collect;
use history::Key;
use super::processes::{Pid, MinimalProcess};
use scan::cgroups::CGroups;


pub struct ReadCache {
    metadata: HashMap<PathBuf, Metadata>,
}

fn get_env_vars(pid: u32) -> (Option<String>, Option<PathBuf>) {
    let file = match File::open(&format!("/proc/{}/environ", pid)) {
        Ok(file) => file,
        Err(e) => {
            debug!("Can't read environ file: {}", e);
            return (None, None);
        }
    };
    let mut buf = BufReader::new(file);
    let mut name = None;
    let mut path = None;
    let mut line = Vec::with_capacity(4096);
    loop {
        line.clear();
        match buf.read_until(0, &mut line) {
            Ok(_) => {}
            Err(e) => {
                debug!("Can't read environ file: {}", e);
                // Assuming file just vanished, i.e. process is dead, so
                // it's useless to return partial data (i.e. name, path)
                return (None, None);
            }
        }
        if line.len() == 0 {
            return (name, path);
        };
        if line.starts_with(b"CANTAL_PATH=") {
            path = Some(PathBuf::from(<OsStr as OsStrExt>::from_bytes(
                &line["CANTAL_PATH=".len()..line.len()-1])));
            if name.is_some() {  // both are ready
                return (name, path);
            }
        }
        if line.starts_with(b"CANTAL_APPNAME=") {
            let val = &line["CANTAL_APPNAME=".len()..line.len()-1];
            if val.is_ascii() {
                name = Some(str::from_utf8(val).unwrap().into());
            } else {
                warn!("Can't decode appname for {}: {:?}", pid, val);
                continue;
            };
            if path.is_some() {  // both are ready
                return (name, path);
            }
        }
    }
    return (name, path);
}

fn relative_from(path: &Path, prefix: &Path) -> PathBuf {
    let mut pref_iter = prefix.components();
    let mut path_iter = path.components();
    loop {
        if let Some(cmp) = pref_iter.next() {
            assert_eq!(Some(cmp), path_iter.next());
        } else {
            return path_iter.as_path().to_path_buf();
        }
    }
}

fn add_suffix<P: AsRef<Path>, E: AsRef<OsStr>>(path: P, ext: E) -> PathBuf
{
    let result: &Path = path.as_ref();
    let mut name = result.file_name().map(OsString::from)
                   .unwrap_or_else(OsString::new);
    name.push(ext);
    result.with_file_name(name)
}

fn read_values(cache: &ReadCache, path: &PathBuf)
    -> (Option<Vec<(Rc<Descriptor>, Value)>>, Option<Metadata>)
{
    let mpath = add_suffix(path, ".meta");
    if let Some(meta) = cache.metadata.get(path) {
        let data = meta.read_data(&add_suffix(path, ".values"));
        if let Err(ref e) = data {
            debug!("Error reading {:?}: {}", mpath, e);
        }
        // TODO(tailhook) check mtime of metadata
        if meta.still_fresh(&mpath) {
            return (data.ok(), None);
        }
    }
    for _ in 0..1 {
        let mres = Metadata::read(&mpath);
        if let Ok(meta) = mres {
            debug!("Read new metadata {:?}", path);
            let data = meta.read_data(&add_suffix(path, ".values"));
            if !meta.still_fresh(&mpath) {
                continue;
            }
            return (data.ok(), Some(meta));
        } else {
            let err = mres.err().unwrap();
            info!("Error reading metadata {:?}: {}", mpath, err);
            return (None, None);
        }
    }
    warn!("Constantly changing metadata {:?}", mpath);
    return (None, None);
}

fn key(pid: &str, cgroup: Option<&str>, name_opt: &Option<String>, json: &Json)
    -> Result<Key, ()>
{
    if let Some(cgrp) = cgroup {
        Key::from_json(json, &[
            ("cgroup", cgrp),
            ("pid", pid),
            ])
    } else if let Some(ref name) = *name_opt {
        Key::from_json(json, &[
            ("appname", name),
            ("pid", pid),
            ])
    } else {
        Key::from_json(json, &[
            ("pid", pid),
            ])
    }
}

pub fn read(tip: &mut Tip, cache: &mut ReadCache, processes: &[MinimalProcess],
    cgroups: &CGroups)
{
    for prc in processes.iter() {
        let (name_opt, path_opt) = get_env_vars(prc.pid);
        if let Some(path) = path_opt {
            let pid = prc.pid.to_string();
            let cgroup = cgroups.get(&prc.pid).map(|x| &x[..]);
            // TODO(tailhook) check if not already visited
            let realpath = Path::new(&format!("/proc/{}/root", prc.pid))
                .join(path.strip_prefix("/").unwrap_or(&path));
            let (data, new_meta) = read_values(cache, &realpath);
            if let Some(data) = data {
                for (desc, value) in data.into_iter() {
                    if let Ok(key) = key(&pid, cgroup, &name_opt, &desc.json) {
                        tip.add(key, value);
                    }
                }
            }
            if let Some(meta) = new_meta {
                cache.metadata.insert(realpath, meta);
            }
        }
    }
}

impl ReadCache {
    pub fn new() -> ReadCache {
        ReadCache {
            metadata: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod test_add_suffix {
    use std::path::{Path, PathBuf};
    use super::add_suffix;

    #[test]
    fn normal() {
        assert_eq!(add_suffix(&Path::new("/hello/world"), ".x"),
                   PathBuf::from("/hello/world.x"));
        assert_eq!(add_suffix(&Path::new("/hello/world.1.2"), ".values"),
                   PathBuf::from("/hello/world.1.2.values"));
        assert_eq!(add_suffix(&Path::new("/hello.1/world"), ".values"),
                   PathBuf::from("/hello.1/world.values"));
    }

    /// This test here is to keep track of ugly behavior of the function
    /// this kind of behavior is not carved in stone and kept here until
    /// we find more clear way to deal with this kind of paths
    #[test]
    fn ugly_behavior() {
        assert_eq!(add_suffix(&Path::new("/hello/world/"), ".values"),
                   PathBuf::from("/hello/world.values"));
    }
}
