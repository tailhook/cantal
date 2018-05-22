use std::io::Read;
use std::fs::File;
use std::net::{SocketAddr};
use std::ptr::{null_mut};
use std::str::FromStr;

use nix;
use nix::errno::Errno;
use nix::unistd::gethostname;
use nix::sys::socket::{InetAddr, sockaddr_in};
use libc::{getifaddrs, freeifaddrs};
use libc::{AF_INET};
use rand;
use rand::RngCore;

use id::Id;


pub fn machine_id() -> Id {
    let mut buf = String::with_capacity(33);
    File::open("/etc/machine-id")
    .and_then(|mut f| f.read_to_string(&mut buf))
    .map_err(|e| error!("Error reading /etc/machine-id: {}", e))
    .and_then(|bytes| if bytes != 32 && bytes != 33  {
        error!("Wrong length of /etc/machine-id");
        Err(())
    } else {
        Id::from_str(&buf[..])
        .map_err(|e| error!("Error decoding /etc/machine-id: {}", e))
    }).unwrap_or_else(|_| {
        let mut res = vec![0u8; 16];
        rand::thread_rng().fill_bytes(&mut res[..]);
        Id::new(res)
    })
}

pub fn hostname() -> Result<String, String> {
    let mut buf = [0u8; 255];
    try!(gethostname(&mut buf)
        .map_err(|e| format!("gethostname: Can't get hostname: {:?}", e)));
    buf.iter().position(|&x| x == 0)
        .ok_or(format!("gethostname: Hostname is not terminated"))
        .and_then(|idx| String::from_utf8(buf[..idx].to_owned())
            .map_err(|e| format!("Can't decode hostname: {}", e)))
}

pub fn my_addresses(port: u16) -> Result<Vec<SocketAddr>, nix::Error> {
    let mut res = Vec::new();
    let mut raw = null_mut();
    unsafe {
        if getifaddrs(&mut raw) != 0 {
            return Err(nix::Error::Sys(Errno::last()));
        }
        let mut cur = raw;
        while cur != null_mut() {
            if (*cur).ifa_addr != null_mut() &&
                (*(*cur).ifa_addr).sa_family == AF_INET as u16
            {
                let a = InetAddr::V4(*((*cur).ifa_addr as *const sockaddr_in));
                res.push(InetAddr::new(a.ip(), port).to_std());
            } // TODO(tailhook) AF_INET6
            cur = (*cur).ifa_next;
        }
        freeifaddrs(raw);
    }
    Ok(res)
}
