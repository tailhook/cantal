use std::ptr;
use std::io;
use std::io::Error;
use std::mem::size_of;
use std::net::TcpListener;
use std::os::unix::io::{AsRawFd, RawFd};

use libc;
use libc::{c_int};

use self::ReadResult as R;
use self::WriteResult as W;

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
    pub const EPOLLHUP: u32 = 0x010;

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

pub struct EPoll(RawFd);

pub enum EPollEvent {
    Input(RawFd),
    Output(RawFd),
    Timeout,
}

pub enum ReadResult {
    Read(usize, usize),
    NoData,
    Closed,
    Fatal(io::Error),
}

pub enum WriteResult {
    Written(usize),
    Again,
    Closed,
    Fatal(io::Error),
}

impl EPoll {
    pub fn new() -> Result<EPoll, io::Error> {
        let fd = unsafe { linux::epoll_create1(linux::EPOLL_CLOEXEC) };
        if fd < 0 {
            return Err(io::Error::last_os_error());
        }
        return Ok(EPoll(fd));
    }

    #[inline]
    fn fd(&self) -> RawFd {
        let &EPoll(fd) = self;
        return fd;
    }

    pub fn add_fd_in(&self, fd: RawFd) {
        let ev = linux::epoll_event {
            events: linux::EPOLLIN,
            data: fd as u64,
        };
        let rc = unsafe { linux::epoll_ctl(self.fd(),
            linux::EPOLL_CTL_ADD, fd, &ev) };
        if rc < 0 {
            // All documented errors are kinda resource limits, or bad
            // file descriptor. Nothing to be handled in runtime.
            panic!("Error adding fd {} to epoll: {}",
                   fd, io::Error::last_os_error());
        }
    }
    pub fn change_to_out(&self, fd: RawFd) {
        let ev = linux::epoll_event {
            events: linux::EPOLLOUT,
            data: fd as u64,
        };
        let rc = unsafe { linux::epoll_ctl(self.fd(),
            linux::EPOLL_CTL_MOD, fd, &ev) };
        if rc < 0 {
            // All documented errors are kinda resource limits, or bad
            // file descriptor. Nothing to be handled in runtime.
            panic!("Error adding fd {} to epoll: {}",
                   fd, io::Error::last_os_error());
        }
    }
    pub fn del_fd(&self, fd: RawFd) {
        let rc = unsafe { linux::epoll_ctl(self.fd(),
            linux::EPOLL_CTL_DEL, fd, ptr::null()) };
        if rc < 0 {
            // All documented errors are kinda resource limits, or bad
            // file descriptor. Nothing to be handled in runtime.
            panic!("Error adding fd {} to epoll: {}",
                   fd, io::Error::last_os_error());
        }
    }
    pub fn next_event(&self, timeout: Option<f64>) -> EPollEvent {
        loop {
            let mut ev = linux::epoll_event { events: 0, data: 0 };
            let timeo = timeout.map(|x| (x*1000.) as c_int).unwrap_or(-1);
            let rc = unsafe { linux:: epoll_wait(self.fd(), &mut ev, 1, timeo) };
            if rc == 1 {
                // Note we never poll both for in and out
                if (ev.events & linux::EPOLLIN) != 0 {
                    return EPollEvent::Input(ev.data as RawFd);
                } else if (ev.events & linux::EPOLLOUT) != 0 {
                    return EPollEvent::Output(ev.data as RawFd);
                } else if (ev.events & linux::EPOLLHUP) != 0 {
                    unimplemented!();
                }
            } else if rc < 0 {
                if Error::last_os_error().raw_os_error() == Some(libc::EINTR)
                {
                    continue;
                }
                panic!("Unexpected error in loop {}",
                       io::Error::last_os_error());
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

pub fn bind_tcp_socket(host: &str, port: u16) -> Result<RawFd, io::Error> {
    let stream = try!(TcpListener::bind(&(host, port)));
    let sfd = stream.as_raw_fd();
    if unsafe { libc::listen(sfd, 1) } < 0 {
        return Err(io::Error::last_os_error());
    }
    let fd = unsafe { libc::dup(stream.as_raw_fd()) };
    if fd < 0 {
        return Err(io::Error::last_os_error());
    }
    // TODO(tailhook) set non-blocking, set cloexec
    return Ok(fd);
}

pub fn accept(fd: RawFd) -> Result<RawFd, io::Error> {
    let mut sockaddr: libc::sockaddr = libc::sockaddr {
        sa_family: 0,
        sa_data: [0u8; 14],
    };
    let mut addrlen: libc::socklen_t = size_of::<libc::sockaddr>() as u32;
    let child = unsafe { libc::accept(fd, &mut sockaddr, &mut addrlen) };
    if child < 0 {
        return Err(io::Error::last_os_error());
    }
    return Ok(child);
}

pub fn read_to_vec(fd: RawFd, vec: &mut Vec<u8>) -> ReadResult {
    let oldlen = vec.len();
    let newend = oldlen + BUFFER_SIZE;
    vec.reserve(BUFFER_SIZE);
    unsafe { vec.set_len(newend) };
    let bytes = unsafe { libc::read(fd,
        (&mut vec[oldlen..newend]).as_mut_ptr() as *mut libc::c_void,
        BUFFER_SIZE as u64) };
    if bytes < 0 {
        unsafe { vec.set_len(oldlen) };
        if Error::last_os_error().raw_os_error() == Some(libc::EAGAIN) {
            return R::NoData;
        } else {
            return R::Fatal(io::Error::last_os_error());
        }
    } else if bytes == 0 {
        return R::Closed;
    } else {
        unsafe { vec.set_len(oldlen + bytes as usize) };
        return R::Read(oldlen, oldlen + bytes as usize);
    }
}

pub fn write(fd: RawFd, chunk: &[u8]) -> WriteResult {
    let bytes = unsafe { libc::write(fd,
        chunk.as_ptr() as *mut libc::c_void,
        chunk.len() as u64) };
    if bytes < 0 {
        if Error::last_os_error().raw_os_error() == Some(libc::EAGAIN) {
            return W::Again;
        } else {
            return W::Fatal(io::Error::last_os_error());
        }
    } else if bytes == 0 {
        return W::Closed;
    } else {
        return W::Written(bytes as usize);
    }
}

pub fn close(fd: RawFd) {
    let rc = unsafe { libc::close(fd) };
    if rc < 0 {
        if Error::last_os_error().raw_os_error() == Some(libc::EINTR) {
            return;
        }
        panic!("Close returned {}", io::Error::last_os_error());
    }
}
