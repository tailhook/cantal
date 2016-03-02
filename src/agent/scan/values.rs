use std::rc::Rc;
use std::io::{BufReader, BufRead};
use std::fs::{File};
use std::ffi::OsStr;
use std::os::unix::prelude::OsStrExt;
use std::path::{Path, PathBuf};
use std::collections::{HashMap};

use cantal::{Metadata, Value, Descriptor};

use super::Tip;
use super::super::util::tree_collect;
use history::Key;
use super::processes::{Pid, MinimalProcess};
use super::super::mountpoints::{MountPrefix, parse_mount_point};


pub struct ReadCache {
    metadata: HashMap<PathBuf, Metadata>,
    mountpoints: HashMap<(i32, i32), Vec<MountPrefix>>,
}

fn get_env_var(pid: u32) -> Option<PathBuf> {
    File::open(&format!("/proc/{}/environ", pid))
    .map(|f| BufReader::new(f))
    .and_then(|mut f| {
        loop {
            let mut line = Vec::with_capacity(4096);
            try!(f.read_until(0, &mut line));
            if line.len() == 0 {
                return Ok(None);
            };
            if line.starts_with(b"CANTAL_PATH=") {
                return Ok(Some(PathBuf::from(<OsStr as OsStrExt>::from_bytes(
                    &line["CANTAL_PATH=".len()..line.len()-1]))));
            }
        }
    })
    .map_err(|e| debug!("Can't read environ file: {}", e))
    .ok()
    .and_then(|opt| opt)
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

fn match_mountpoint(cache: &ReadCache, pid: Pid, path: &Path)
    -> Result<PathBuf, ()>
{
    let mut best_match = None;
    let mut file = BufReader::new(try!(
        File::open(&format!("/proc/{}/mountinfo", pid))
        .map_err(|e| debug!("Error reading mountinfo: {}", e))));
    loop {
        let mut line = String::with_capacity(256);
        try!(file.read_line(&mut line)
            .map_err(|e| error!("Error reading mountinfo: {}", e)));
        if line.len() == 0 { break; }
        let mp = try!(parse_mount_point(&line)
            .map_err(|()| error!("Error parsing mount point: {:?}", line)));
        if path.starts_with(mp.mounted_at) {
            if let Some((ref mut pref, ref mut pt, ref mut dev)) = best_match {
                // Modify only if new path is longer
                if Path::new(mp.mounted_at).starts_with(&pt) {
                    *pref = PathBuf::from(mp.prefix);
                    *pt = PathBuf::from(mp.mounted_at);
                    *dev = mp.device_id;
                }
            } else {
                best_match = Some((
                    PathBuf::from(mp.prefix),
                    PathBuf::from(mp.mounted_at),
                    mp.device_id));
            }
        }
    }
    let (prefix, mountpoint, device) = try!(best_match.ok_or(()));
    let suffix = prefix.join(&relative_from(&path, &mountpoint));
    if let Some(ref mprefixes) = cache.mountpoints.get(&device) {
        for pref in mprefixes.iter() {
            if Path::new(&pref.prefix) == Path::new("/") ||
                suffix.starts_with(&pref.prefix)
            {
                // TODO(tailhook) check name_to_handle_at
                return Ok(pref.mounted_at.join(
                    &relative_from(&suffix, &pref.prefix)));
            }
        }
    }
    info!("Can't find mountpoint for \
           dev: {:?}, pid: {}, prefix: {:?}, path: {:?}",
        device, pid, prefix, path);
    return Err(());
}

fn read_values(cache: &ReadCache, path: &PathBuf)
    -> (Option<Vec<(Rc<Descriptor>, Value)>>, Option<Metadata>)
{
    let mpath = path.with_extension("meta");
    if let Some(meta) = cache.metadata.get(path) {
        let data = meta.read_data(&path.with_extension("values"));
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
            let data = meta.read_data(&path.with_extension("values"));
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

pub fn read(tip: &mut Tip, cache: &mut ReadCache, processes: &[MinimalProcess])
{
    for prc in processes.iter() {
        if let Some(path) = get_env_var(prc.pid) {
            // TODO(tailhook) check if not already visited
            if let Ok(realpath) = match_mountpoint(cache, prc.pid, &path) {
                let (data, new_meta) = read_values(cache, &realpath);
                if let Some(data) = data {
                    for (desc, value) in data.into_iter() {
                        let pid = &format!("{}", prc.pid);
                        if let Ok(key) = Key::from_json(
                            &desc.json, &[("pid", pid)])
                        {
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
}

fn parse_mountpoints() -> Result<HashMap<(i32, i32), Vec<MountPrefix>>, ()> {
    let mut tmp = vec!();
    let mut file = BufReader::new(try!(File::open("/proc/self/mountinfo")
        .map_err(|e| error!("Error reading mountinfo: {}", e))));
    loop {
        let mut line = String::with_capacity(256);
        try!(file.read_line(&mut line)
            .map_err(|e| error!("Error reading mountinfo: {}", e)));
        if line.len() == 0 { break; }
        let mp = try!(parse_mount_point(&line)
            .map_err(|()| error!("Error parsing mount point: {:?}", line)));
        tmp.push((mp.device_id, MountPrefix::from_mount_point(&mp)));
    }
    return Ok(tree_collect(tmp.into_iter()));
}

impl ReadCache {
    pub fn new() -> ReadCache {
        ReadCache {
            metadata: HashMap::new(),
            mountpoints: parse_mountpoints().unwrap(),
        }
    }
}
