use std::io::{Read, Error};
use std::mem::transmute;
use std::io::ErrorKind::Other;

pub trait ReadHostBytes {
    fn read_u64(&mut self) -> Result<u64, Error>;
    fn read_i64(&mut self) -> Result<i64, Error>;
    fn read_f64(&mut self) -> Result<f64, Error>;
    fn read_bytes(&mut self, num: usize) -> Result<Vec<u8>, Error>;
    fn read_chunk(&mut self, num: usize) -> Result<Vec<u8>, Error>;
}

impl<I> ReadHostBytes for I where I: Read
{
    fn read_u64(&mut self) -> Result<u64, Error> {
        let mut buf = [0u8; 8];
        let mut read = 0;
        while read < 8 {
            let chunk = try!(self.read(&mut buf[read..]));
            if chunk == 0 {
                return Err(Error::new(Other, "Not enough bytes", None));
            }
            read += chunk;
        }
        return Ok(unsafe { *(buf.as_ptr() as *const u64) });
    }
    fn read_i64(&mut self) -> Result<i64, Error> {
        let mut buf = [0u8; 8];
        let mut read = 0;
        while read < 8 {
            let chunk = try!(self.read(&mut buf[read..]));
            if chunk == 0 {
                return Err(Error::new(Other, "Not enough bytes", None));
            }
            read += chunk;
        }
        return Ok(unsafe { *(buf.as_ptr() as *const i64) });
    }
    fn read_f64(&mut self) -> Result<f64, Error> {
        let mut buf = [0u8; 8];
        let mut read = 0;
        while read < 8 {
            let chunk = try!(self.read(&mut buf[read..]));
            if chunk == 0 {
                return Err(Error::new(Other, "Not enough bytes", None));
            }
            read += chunk;
        }
        return Ok(unsafe { *(buf.as_ptr() as *const f64) });
    }
    fn read_bytes(&mut self, num: usize) -> Result<Vec<u8>, Error> {
        let mut buf = Vec::with_capacity(num);
        unsafe { buf.set_len(num) };
        let mut read = 0;
        while read < 8 {
            let chunk = try!(self.read(&mut buf[read..]));
            if chunk == 0 {
                return Err(Error::new(Other, "Not enough bytes", None));
            }
            read += chunk;
        }
        return Ok(buf);
    }
    fn read_chunk(&mut self, num: usize) -> Result<Vec<u8>, Error> {
        let mut buf = Vec::with_capacity(num);
        unsafe { buf.set_len(num) };
        let chunk = try!(self.read(&mut buf[..]));
        unsafe { buf.set_len(chunk) };
        return Ok(buf);
    }
}

