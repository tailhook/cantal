use std::ptr;
use std::fmt::String;
use libc;

pub mod machine;
pub mod processes;


extern {
    fn gettimeofday(tp: *mut libc::timeval, tzp: *mut libc::c_void)
        -> libc::c_int;
}

pub fn time_ms() -> u64 {
    let mut tv = libc::timeval { tv_sec: 0, tv_usec: 0 };
    unsafe { gettimeofday(&mut tv, ptr::null_mut()) };
    return (tv.tv_sec as u64)*1000 +  (tv.tv_usec as u64) / 1000;
}
