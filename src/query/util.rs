use std::ffi::{CString, IntoBytes};
use std::path::Path;
use std::io::Error as IoError;
use std::mem::zeroed;
use std::os::unix::AsRawFd;
use libc::{stat, dev_t, off_t, time_t, c_long, ino_t};
use libc::funcs::posix88::stat_::stat as stat_path;
use libc::{fstat};


#[derive(PartialEq, Eq)]
pub struct Stat {
    dev: dev_t,
    inode: ino_t,
    size: off_t,
    mtime: (time_t, c_long),
}


pub fn file_stat<F: AsRawFd>(file: &F) -> Result<Stat, IoError> {
    let mut stat;
    unsafe {
        stat = zeroed();
        if fstat(file.as_raw_fd(), &mut stat) != 0 {
            return Err(IoError::last_os_error());
        }
    };
    return Ok(Stat {
        dev: stat.st_dev,
        inode: stat.st_ino,
        size: stat.st_size,
        mtime: (stat.st_mtime, stat.st_mtime_nsec),
    });
}

pub fn path_stat(path: &Path) -> Result<Stat, IoError> {
    let mut stat;
    unsafe {
        stat = zeroed();
        if stat_path(CString::new(path.to_str().unwrap())
                     .unwrap().as_bytes().as_ptr() as *const i8,
                     &mut stat) != 0 {
            return Err(IoError::last_os_error());
        }
    };
    return Ok(Stat {
        dev: stat.st_dev,
        inode: stat.st_ino,
        size: stat.st_size,
        mtime: (stat.st_mtime, stat.st_mtime_nsec),
    });
}
