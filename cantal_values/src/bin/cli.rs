extern crate argparse;
extern crate cantal_values;

use std::fmt::Display;
use std::path::{Path, PathBuf};
use std::io::{stderr, Write, Read, BufRead, BufReader};
use std::error::Error;
use std::ffi::OsStr;
use std::str::{FromStr, from_utf8};
use std::process::exit;
use std::os::unix::ffi::OsStrExt;
use std::fs::{File, read_link, read_dir};

use argparse::{ArgumentParser, ParseList};

use cantal_values::Metadata;


fn read_file<D: Display>(prefix: D, path: &Path) -> Result<(), Box<Error>> {
    let meta = try!(Metadata::read(&path.with_extension("meta")));
    let data = try!(meta.read_data(&path));
    for &(ref descr, ref item) in data.iter() {
        println!("{}: {} {:?}", prefix, descr.textname, item);
    }
    Ok(())
}

fn read_from_pid(pid: u32) -> Result<(), Box<Error>> {
    let mut file = BufReader::new(try!(
        File::open(format!("/proc/{}/environ", pid))));
    let mut cantal_path = None;
    let mut xdg_runtime_dir = None;
    let mut buf = Vec::with_capacity(1024);
    loop {
        buf.clear();
        let bytes = try!(file.read_until(0, &mut buf));
        if bytes == 0 { break }
        if buf[..bytes].starts_with(b"CANTAL_PATH=") {
            cantal_path = Some(buf[12..bytes].to_owned());
        }
        if buf[..bytes].starts_with(b"XDG_RUNTIME_DIR=") {
            xdg_runtime_dir = Some(buf[16..bytes].to_owned());
        }
    }
    drop(file);
    // The same logic as in cantal-py and cantal-go
    if let Some(mut bytes) = cantal_path {
        bytes.extend(b".values");
        let path = Path::new(OsStr::from_bytes(&bytes));
        try!(read_file(pid, path));
    } else {
        let prefix = if let Some(ref bytes) = xdg_runtime_dir {
            Path::new(OsStr::from_bytes(bytes))
        } else {
            Path::new("/tmp")
        };
        // We don't know process pid in namespace so we just scan
        // open files to find matching file.
        for entry in try!(read_dir(format!("/proc/{}/fd", pid))) {
            let path = try!(entry).path();
            let path = try!(read_link(&path));
            if path.starts_with(prefix) &&
                path.extension() == Some(OsStr::new("values"))
            {
                try!(read_file(pid,
                    &Path::new(&format!("/proc/{}/root", pid))
                    .join(path.strip_prefix("/").unwrap())));
            }
        }
    }
    Ok(())
}

fn main() {
    let mut files = Vec::<PathBuf>::new();
    {
        let mut ap = ArgumentParser::new();
        ap.refer(&mut files)
            .add_argument("file_or_pid", ParseList, "Pid of the process,
            or file name of the `.values` file, or list of files.");
        ap.parse_args_or_exit();
    }
    let mut retcode = 0;
    for f in files.iter() {
        if f.exists() {
            if let Err(e) = read_file(f.display(), &f) {
                writeln!(&mut stderr(),
                    "Error reading values at {:?}: {}", f, e).ok();
                retcode = 1;
            }
        } else {
            let maybepid = from_utf8(f.as_os_str().as_bytes()).ok()
                           .and_then(|x| u32::from_str(x).ok());
            let pid = match maybepid {
                Some(x) => x,
                None => {
                    writeln!(&mut stderr(),
                        "No file {:?} exists, and it's not a pid", f).ok();
                    retcode = 1;
                    continue;
                }
            };

            if let Err(e) = read_from_pid(pid) {
                writeln!(&mut stderr(),
                    "Error reading for pid {}: {}", pid, e).ok();
                retcode = 1;
            }
        }
    }
    exit(retcode);
}
