use std::ptr;
use std::io::IoError;
use std::os::errno;
use std::mem::size_of;
use std::io::net::tcp::TcpListener;
use std::os::unix::{AsRawFd, Fd};

use libc;
use libc::{c_int};

use self::ReadResult::*;

const BUFFER_SIZE: usize = 4096;

mod linux {
    use libc::{c_int, c_void};
    use libc::{close};

    pub const EPOLL_CLOEXEC: c_int = 0x80000;
    pub const EPOLL_CTL_ADD: c_int = 1;
    pub const EPOLL_CTL_DEL: c_int = 2;
    pub const EPOLL_CTL_MOD: c_int = 3;
    pub const EPOLLIN: u32 = 0x001;
    pub const EPOLLPRI: u32 =  0x002;
    pub const EPOLLOUT: u32 = 0x004;

    #[repr(C)]
    pub struct epoll_event {
        pub events: u32,
        pub data: u64,   // in fact this is u64-sized union
    }

    extern "C" {
        pub fn epoll_create1(flags: c_int) -> c_int;
        pub fn epoll_ctl(epfd: c_int, op: c_int, fd: c_int,
                         event: *const epoll_event) -> c_int;
        pub fn epoll_wait(epfd: c_int, events: *mut epoll_event,
                          maxevents: c_int, timeout: c_int) -> c_int;
    }
}

pub struct EPoll(Fd);

pub enum EPollEvent {
    Input(Fd),
    Timeout,
}

pub enum ReadResult {
    Read(usize, usize),
    NoData,
    Closed,
    Fatal(IoError),
}

impl EPoll {
    pub fn new() -> Result<EPoll, IoError> {
        let fd = unsafe { linux::epoll_create1(linux::EPOLL_CLOEXEC) };
        if fd < 0 {
            return Err(IoError::last_error());
        }
        return Ok(EPoll(fd));
    }

    #[inline]
    fn fd(&self) -> Fd {
        let &EPoll(fd) = self;
        return fd;
    }

    pub fn add_fd_in(&self, fd: Fd) -> Result<(), IoError> {
        let ev = linux::epoll_event {
            events: linux::EPOLLIN,
            data: fd as u64,
        };
        let rc = unsafe { linux::epoll_ctl(self.fd(),
            linux::EPOLL_CTL_ADD, fd, &ev) };
        if rc < 0 {
            return Err(IoError::last_error());
        }
        Ok(())
    }
    pub fn del_fd(&self, fd: Fd) -> Result<(), IoError> {
        let rc = unsafe { linux::epoll_ctl(self.fd(),
            linux::EPOLL_CTL_DEL, fd, ptr::null()) };
        if rc < 0 {
            return Err(IoError::last_error());
        }
        Ok(())
    }
    pub fn next_event(&self, timeout: Option<f64>) -> EPollEvent {
        loop {
            let mut ev = linux::epoll_event { events: 0, data: 0 };
            let timeo = timeout.map(|x| (x*1000.) as c_int).unwrap_or(-1);
            let rc = unsafe { linux:: epoll_wait(self.fd(), &mut ev, 1, timeo) };
            if rc == 1 {
                return EPollEvent::Input(ev.data as Fd);
            } else if rc < 0 {
                if errno() == libc::EINTR as usize {
                    continue;
                }
                panic!("Unexpected error in loop {}", IoError::last_error());
            }
        }
    }
}

impl Drop for EPoll {
    fn drop(&mut self) {
        unsafe {
            libc::close(self.fd());
        }
    }
}

pub fn bind_tcp_socket(host: &str, port: u16) -> Result<Fd, IoError> {
    let stream = try!(TcpListener::bind((host, port)));
    let sfd = stream.as_raw_fd();
    if unsafe { libc::listen(sfd, 1) } < 0 {
        return Err(IoError::last_error());
    }
    let fd = unsafe { libc::dup(stream.as_raw_fd()) };
    if fd < 0 {
        return Err(IoError::last_error());
    }
    // TODO(tailhook) set non-blocking, set cloexec
    return Ok(fd);
}

pub fn accept(fd: Fd) -> Result<Fd, IoError> {
    let mut sockaddr: libc::sockaddr = libc::sockaddr {
        sa_family: 0,
        sa_data: [0u8; 14],
    };
    let mut addrlen: libc::socklen_t = size_of::<libc::sockaddr>() as u32;
    let child = unsafe { libc::accept(fd, &mut sockaddr, &mut addrlen) };
    if child < 0 {
        return Err(IoError::last_error());
    }
    return Ok(child);
}

pub fn read_to_vec(fd: Fd, vec: &mut Vec<u8>) -> ReadResult {
    let oldlen = vec.len();
    let newend = oldlen + BUFFER_SIZE;
    vec.reserve(BUFFER_SIZE);
    unsafe { vec.set_len(newend) };
    let bytes = unsafe { libc::read(fd,
        vec.slice_mut(oldlen, newend).as_mut_ptr() as *mut libc::c_void,
        BUFFER_SIZE as u64) };
    if bytes < 0 {
        unsafe { vec.set_len(oldlen) };
        if errno() == libc::EAGAIN as usize {
            return NoData;
        } else {
            return Fatal(IoError::last_error());
        }
    } else if bytes == 0 {
        return Closed;
    } else {
        unsafe { vec.set_len(oldlen + bytes as usize) };
        return Read(oldlen, oldlen + bytes as usize);
    }
}

pub fn close(fd: Fd) {
    let rc = unsafe { libc::close(fd) };
    if rc < 0 {
        if errno() == libc::EINTR as usize {
            return;
        }
        panic!("Close returned {}", IoError::last_error());
    }
}
