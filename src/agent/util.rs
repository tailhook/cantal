use std::io::Error as IoError;
use std::ptr::copy;
use std::io::{Read, Write};
use std::io::ErrorKind::{Interrupted, WouldBlock};
use std::cmp::min;
use std::hash::{Hash};
use std::collections::HashMap;

const BUFFER_SIZE: usize = 4096;


pub fn tree_collect<K: Hash + Eq, V, I: Iterator<Item=(K, V)>>(iter: I)
    -> HashMap<K, Vec<V>>
{
    let mut result = HashMap::new();
    for (k, v) in iter {
        if let Some(vec) = result.get_mut(&k) {
            let mut val: &mut Vec<V> = vec;
            val.push(v);
            continue;
        }
        result.insert(k, vec!(v));
    }
    return result;
}

pub enum ReadVec {
    Full,
    More,
    Wait,
    Close,
    Error(IoError),
}

impl ReadVec {
    pub fn read<R: Read>(stream: &mut R, buf: &mut Vec<u8>, max: usize)
        -> ReadVec
    {
        let mut cap = buf.capacity();
        let len = buf.len();
        if cap < max {
            if len == 0 {
                buf.reserve(BUFFER_SIZE);
                cap = buf.capacity();
            } else if cap - len < len*2 {
                buf.reserve(min(len, max - len));
                cap = buf.capacity();
            }
        }
        let maxlen = min(cap, max);
        let nlen;
        let res;
        unsafe {
            buf.set_len(maxlen);
            res = stream.read(&mut buf[len..maxlen]);
            nlen = len + *res.as_ref().unwrap_or(&0);
            buf.set_len(nlen);
        }
        match res {
            Ok(0) => ReadVec::Close,
            Ok(_) => {
                if nlen < max {
                    ReadVec::More
                } else {
                    ReadVec::Full
                }
            }
            Err(ref e) if e.kind() == Interrupted || e.kind() == WouldBlock
            => ReadVec::Wait,
            Err(e) => ReadVec::Error(e),
        }
    }
}

pub enum WriteVec {
    Done,
    More(Vec<u8>),
    Close,
    Error(IoError),
}

impl WriteVec {
    pub fn write<W: Write>(stream: &mut W, mut buf: Vec<u8>)
        -> WriteVec
    {
        if buf.len() == 0 {
            debug!("Empty output buffer");
            return WriteVec::Done;
        }
        let res = stream.write(&buf);
        match res {
            Ok(0) => WriteVec::Close,
            Ok(x) => {
                if buf.len() > x {
                    buf.consume(x);
                    WriteVec::More(buf)
                } else {
                    WriteVec::Done
                }
            }
            Err(ref e) if e.kind() == Interrupted || e.kind() == WouldBlock
            => WriteVec::More(buf),
            Err(e) => WriteVec::Error(e),
        }
    }
}

pub trait Consume {
    fn consume(&mut self, at: usize);
}

impl<T:Sized> Consume for Vec<T> {
    fn consume(&mut self, at: usize) {
        let len = self.len();
        if at >= len {
            self.truncate(0);
        } else {
            unsafe {
                copy(self[at..len].as_ptr(),
                     self[..len - at].as_mut_ptr(), len - at);
                self.truncate(len - at);
            }
        }
    }
}
