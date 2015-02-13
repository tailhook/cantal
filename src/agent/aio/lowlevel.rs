use std::io::IoError;
use std::os::errno;
use std::io::net::tcp::TcpListener;
use std::os::unix::{AsRawFd, Fd};

use libc;
use libc::{c_int};


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
    pub fn next_event(&self, timeout: Option<f64>) -> EPollEvent {
        loop {
            let mut ev = linux::epoll_event { events: 0, data: 0 };
            let timeo = timeout.map(|x| (x*1000.) as c_int).unwrap_or(-1);
            let rc = unsafe { linux:: epoll_wait(self.fd(), &mut ev, 1, timeo) };
            println!("EPOLL {} {}", ev.events, ev.data);
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
