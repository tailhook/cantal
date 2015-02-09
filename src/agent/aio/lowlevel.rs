use std::io::IoError;
use std::os::unix::Fd;


mod linux {
    use libc::{c_int};
    use libc::{close};

    pub const EPOLL_CLOEXEC: c_int = 0x80000;

    extern "C" {
        pub fn epoll_create1(flags: c_int) -> c_int;
    }
}



pub fn create_epoll() -> Result<Fd, IoError> {
    let fd = unsafe { linux::epoll_create1(linux::EPOLL_CLOEXEC) };
    if fd < 0 {
        return Err(IoError::last_error());
    }
    return Ok(fd);
}
