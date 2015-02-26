use std::str::FromStr;
use std::path::{Path, PathBuf};
use cantal::itertools::{NextStr, NextValue};


#[derive(Debug)]
pub struct MountPrefix {
    pub mount_id: i32,
    pub device_id: (i32, i32),
    pub prefix: PathBuf,
    pub mounted_at: PathBuf,
}

#[derive(Debug)]
pub struct MountPoint<'a> {
    pub mount_id: i32,
    pub device_id: (i32, i32),
    pub prefix: &'a str,
    pub mounted_at: &'a str,
}

fn parse_pair<A:FromStr, B:FromStr>(val: &str) -> Result<(A, B), ()> {
    let mut iter = val.splitn(1, ':');
    return Ok((try!(iter.next_value()), try!(iter.next_value())));
}

pub fn parse_mount_point(line: &str) -> Result<MountPoint, ()> {
    let mut words = line.words();
    Ok(MountPoint {
        mount_id: try!(words.next_value()),
        device_id: try!(words.nth_str(1).and_then(parse_pair)),
        prefix: try!(words.next_str()),
        mounted_at: try!(words.next_str()),
    })
}

impl MountPrefix {
    pub fn from_mount_point(mp: &MountPoint) -> MountPrefix {
        return MountPrefix {
            mount_id: mp.mount_id,
            device_id: mp.device_id,
            prefix: PathBuf::new(mp.prefix),
            mounted_at: PathBuf::new(mp.mounted_at),
        }
    }
}