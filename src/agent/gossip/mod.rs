mod errors;

use std::net::SocketAddr;

use tk_easyloop;
use quick_error::ResultExt;
use tokio_core::net::UdpSocket;


pub use self::errors::InitError;

pub struct Config {
    pub host: String,
    pub port: u16,
}


pub fn spawn(cfg: Config) -> Result<(), InitError> {
    let addr = SocketAddr::new(cfg.host.parse().context(&cfg.host)?, cfg.port);
    let server = UdpSocket::bind(&addr, &tk_easyloop::handle()).context(addr)?;
    Ok(())
}
