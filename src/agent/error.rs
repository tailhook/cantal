use std::io;
use std::net::AddrParseError;

use mio;


pub enum Error {
    Io(io::Error),
    Host(AddrParseError),
    Timer(mio::TimerError),
}

impl From<AddrParseError> for Error {
    fn from(err: AddrParseError) -> Error { Error::Host(err) }
}
impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error { Error::Io(err) }
}
impl From<mio::TimerError> for Error {
    fn from(err: mio::TimerError) -> Error { Error::Timer(err) }
}
impl ::std::fmt::Display for Error {
    fn fmt(&self, fmt: &mut ::std::fmt::Formatter)
        -> Result<(), ::std::fmt::Error>
    {
        match self {
            &Error::Io(ref e) => write!(fmt, "Io error: {}", e),
            &Error::Host(_) => write!(fmt, "Error parsing host to listen to"),
            &Error::Timer(_) => write!(fmt, "Timer adding timer"),
        }
    }
}
impl ::std::fmt::Debug for Error {
    fn fmt(&self, fmt: &mut ::std::fmt::Formatter)
        -> Result<(), ::std::fmt::Error>
    {
        match self {
            &Error::Io(ref e) => write!(fmt, "IoError({:?})", e),
            &Error::Host(_) => write!(fmt, "std::net::AddrParseError"),
            &Error::Timer(_) => write!(fmt, "mio::TimerError"),
        }
    }
}
impl ::std::error::Error for Error {
    fn description(&self) -> &'static str {
        "Error initializing P2P loop"
    }
}
