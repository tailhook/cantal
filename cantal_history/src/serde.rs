use std::io::{Cursor, Read, Write};
use std::collections::VecDeque;

use cbor::{Encoder, Decoder, Config};
use cbor::types::{Tag, Type};
use byteorder::{WriteBytesExt, ReadBytesExt, BigEndian};

use {History, Key};
use super::backlog::Inner;

macro_rules! try_log {
    ($x: expr) => {
        try!(($x).map_err(|e| warn!("Error reading history: {}", e)));
    }
}

macro_rules! read_assert {
    ($x: expr) => {
        if !($x) {
            warn!("Error reading history: {}", stringify!(e));
            return Err(());
        }
    }
}

fn type_len(w: &mut Vec<u8>, t: Type, x: u64) {
    match x {
        0...23                => w.write_u8(t.major() << 5 | x as u8).unwrap(),
        24...0xFF             => w.write_u8(t.major() << 5 | 24).and(w.write_u8(x as u8)).unwrap(),
        0x100...0xFFFF        => w.write_u8(t.major() << 5 | 25).and(w.write_u16::<BigEndian>(x as u16)).unwrap(),
        0x100000...0xFFFFFFFF => w.write_u8(t.major() << 5 | 26).and(w.write_u32::<BigEndian>(x as u32)).unwrap(),
        _                     => w.write_u8(t.major() << 5 | 27).and(w.write_u64::<BigEndian>(x)).unwrap(),
    }
}


fn write_bytes<F>(e: Encoder<Vec<u8>>, num: usize, mut fun: F)
    -> Encoder<Vec<u8>>
    where F: FnMut(&mut Vec<u8>)
{
    let mut v = e.into_writer();
    type_len(&mut v, Type::Bytes, num as u64);
    let oldl = v.len();
    fun(&mut v);
    debug_assert!(oldl + num == v.len());
    Encoder::new(v)
}

fn validate_key(val: &[u8]) -> Result<(), &'static str> {
    let mut d = Decoder::new(Config::default(), Cursor::new(val));
    let num = try!(d.object().map_err(|_| "Invalid key"));
    for _ in 0..num {
        // TODO(tailhook) other types may work in future
        try!(d.text().map_err(|_| "Invalid key"));
        try!(d.text().map_err(|_| "Invalid key"));
    }
    if d.into_reader().position() as usize != val.len() {
        return Err("Invalid key: extra data");
    }
    return Ok(());
}

impl History {
    pub fn encode(&self) -> Box<[u8]> {
        let mut e = Encoder::new(Vec::new());
        e.array(2).unwrap();
        e.object(1).unwrap();
        e.text("version").unwrap();
        e.u8(2).unwrap();
        e.object(2).unwrap();

        e.text("fine").unwrap();
        e.object(3).unwrap();
        e.text("age").unwrap();
        e.u64(self.fine.age).unwrap();

        e.text("timestamps").unwrap();
        e.array(self.fine.timestamps.len()).unwrap();
        for pair in self.fine.timestamps.iter() {
            e.array(2).unwrap();
            e.u64(pair.0).unwrap();
            e.u32(pair.1).unwrap();
        }

        e.text("values").unwrap();
        e.object(self.fine.values.len()).unwrap();
        for (k, v) in self.fine.values.iter() {
            use super::backlog::Value::*;
            e.bytes(k.as_bytes()).unwrap();
            // 27 -- Serialised language-independent object with type name
            // and constructor arguments
            e.tag(Tag::Unassigned(27)).unwrap();
            e.array(4).unwrap(); // Type, tip, age, buf
            match v {
                &Counter(ref v) => {
                    e.text("Counter").unwrap();
                    e.u64(v.tip()).unwrap();
                    e.u64(v.age()).unwrap();
                    e = write_bytes(e, v.buf().bytes().len(), |buf| {
                        buf.extend(v.buf().bytes().cloned());
                    });
                }
                &Integer(ref v) => {
                    e.text("Integer").unwrap();
                    e.i64(v.tip()).unwrap();
                    e.u64(v.age()).unwrap();
                    e = write_bytes(e, v.buf().bytes().len(), |buf| {
                        buf.extend(v.buf().bytes().cloned());
                    });
                }
                &Float(ref v) => {
                    e.text("Float").unwrap();
                    e.f64(v.tip()).unwrap();
                    e.u64(v.age()).unwrap();
                    e = write_bytes(e, v.buf().len()*4, |buf| {
                        for val in v.buf().iter() {
                            buf.write_f64::<BigEndian>(*val).unwrap();
                        }
                    });
                }
            }
        }

        e.text("tip").unwrap();
        e.object(2).unwrap();
        e.text("latest_timestamp").unwrap();
        e.array(2).unwrap();
        e.u64(self.tip.latest_timestamp.0).unwrap();
        e.u32(self.tip.latest_timestamp.1).unwrap();
        e.text("values").unwrap();
        for (k, &(ts, ref v)) in self.tip.values.iter() {
            use values::Value::*;
            e.bytes(k.as_bytes()).unwrap();
            e.array(2).unwrap();
            e.u64(ts).unwrap();
            // 27 -- Serialised language-independent object with type name
            // and constructor arguments
            e.tag(Tag::Unassigned(27)).unwrap();
            match v {
                &Counter(v) => {
                    e.array(2).unwrap(); // Type, value
                    e.text("Counter").unwrap();
                    e.u64(v).unwrap();
                }
                &Integer(v) => {
                    e.array(2).unwrap(); // Type, value
                    e.text("Integer").unwrap();
                    e.i64(v).unwrap();
                }
                &Float(v) => {
                    e.array(2).unwrap(); // Type, value
                    e.text("Float").unwrap();
                    e.f64(v).unwrap();
                }
                &State(ts, ref val) => {
                    e.array(3).unwrap(); // Type, value
                    e.text("State").unwrap();
                    e.u64(ts).unwrap();
                    e.text(val).unwrap();
                }
            }
        }


        e.into_writer().into_boxed_slice()
    }
    pub fn decode<R:Read>(stream: R) -> Result<History, ()> {
        let mut d = Decoder::new(Config::default(), stream);
        let tlen = try_log!(d.array());
        read_assert!(tlen == 2);
        let volen = try_log!(d.object());
        read_assert!(volen == 1);
        read_assert!(&try_log!(d.text())[..] == "version");
        read_assert!(try_log!(d.u8()) == 2);
        read_assert!(try_log!(d.object()) == 2);

        let mut h = History::new();
        match &try_log!(d.text())[..] {
            "fine" => {
                let plen = try_log!(d.object());
                read_assert!(plen == 3);
                // We temporarily rely on exact order of properties
                read_assert!(&try_log!(d.text())[..] == "age");
                let age = try_log!(d.u64());
                read_assert!(&try_log!(d.text())[..] == "timestamps");
                let tslen = try_log!(d.array());
                for _ in 0..tslen {
                    let plen = try_log!(d.array());
                    read_assert!(plen == 2);
                    h.fine.timestamps.push_back(
                        (try_log!(d.u64()), try_log!(d.u32())));
                }

                let olen = try_log!(d.object());
                for _ in 0..olen {
                    use super::backlog::Value::*;
                    let key_bytes = try_log!(d.bytes());
                    try_log!(validate_key(&key_bytes));
                    let tag = try_log!(d.tag());
                    read_assert!(tag == Tag::Unassigned(27));
                    let arlen = try_log!(d.array());
                    read_assert!(arlen == 4);
                    match &try_log!(d.text())[..] {
                        "Counter" => {
                            let tip = try_log!(d.u64());
                            let age = try_log!(d.u64());
                            let deltas = try_log!(d.bytes());
                            h.fine.values.insert(
                                Key(key_bytes.into_boxed_slice()),
                                Counter(Inner::unpack(tip, age, deltas)));
                        }
                        "Integer" => {
                            let tip = try_log!(d.i64());
                            let age = try_log!(d.u64());
                            let deltas = try_log!(d.bytes());
                            h.fine.values.insert(
                                Key(key_bytes.into_boxed_slice()),
                                Integer(Inner::unpack(tip, age, deltas)));
                        }
                        "Float" => {
                            let tip = try_log!(d.f64());
                            let age = try_log!(d.u64());
                            let deltas = try_log!(d.bytes());
                            let mut deque = VecDeque::new();
                            let num = deltas.len() / 4;
                            read_assert!(num % 4 == 0);
                            let mut cur = Cursor::new(deltas);
                            for _ in 0..num {
                                deque.push_back(
                                    cur.read_f64::<BigEndian>().unwrap());
                            }
                            h.fine.values.insert(
                                Key(key_bytes.into_boxed_slice()),
                                Float(Inner::unpack(tip, age, deque)));
                        }
                        _ => {
                            error!("Wrong data type in fine timestamps");
                            return Err(());
                        }
                    }
                }
            }
            "tip" => {
                let plen = try_log!(d.object());
                read_assert!(plen == 2);
                // We temporarily rely on exact order of properties
                read_assert!(&try_log!(d.text())[..] == "latest_timestamp");
                read_assert!(try_log!(d.array()) == 2);
                h.tip.latest_timestamp = (try_log!(d.u64()),
                                          try_log!(d.u32()));
                let olen = try_log!(d.object());
                for _ in 0..olen {
                    use values::Value::*;
                    let key_bytes = try_log!(d.bytes());
                    try_log!(validate_key(&key_bytes));

                    read_assert!(try_log!(d.array()) == 2);
                    let ts = try_log!(d.u64());
                    let tag = try_log!(d.tag());
                    read_assert!(tag == Tag::Unassigned(27));
                    let arlen = try_log!(d.array());
                    match &try_log!(d.text())[..] {
                        "Counter" => {
                            read_assert!(arlen == 2);
                            h.tip.values.insert(
                                Key(key_bytes.into_boxed_slice()),
                                (ts, Counter(try_log!(d.u64()))));
                        }
                        "Integer" => {
                            read_assert!(arlen == 2);
                            h.tip.values.insert(
                                Key(key_bytes.into_boxed_slice()),
                                (ts, Integer(try_log!(d.i64()))));
                        }
                        "Float" => {
                            read_assert!(arlen == 2);
                            h.tip.values.insert(
                                Key(key_bytes.into_boxed_slice()),
                                (ts, Float(try_log!(d.f64()))));
                        }
                        "State" => {
                            read_assert!(arlen == 3);
                            h.tip.values.insert(
                                Key(key_bytes.into_boxed_slice()),
                                (ts, State(try_log!(d.u64()),
                                           try_log!(d.text()))));
                        }
                        _ => {
                            error!("Wrong data type in tip timestamps");
                            return Err(());
                        }
                    }
                }
            }
            x => {
                error!("Extra data collection: {:?}", x);
                return Err(());
            }
        }
        Ok(h)
    }
}

#[cfg(test)]
mod test {
    use std::io::Cursor;
    use {History, Key, ValueSet};
    use values::Value::{Counter, State};
    use std::collections::{HashMap, HashSet};


    #[test]
    fn roundtrip() {
        let mut h = History::new();
        h.fine.push((1000, 10), vec![
            (&Key::metric("test1"), &Counter(10)),
            (&Key::metric("test2"), &Counter(20)),
        ].into_iter());
        h.fine.push((2000, 10), vec![
            (&Key::metric("test2"), &Counter(20)),
            (&Key::metric("test3"), &Counter(30)),
        ].into_iter());
        h.tip.push((2000, 10), vec![
            (&Key::metric("st1"), &State(1500, "hello".to_string())),
            (&Key::metric("st2"), &Counter(30)),
        ].into_iter());
        let h = History::decode(Cursor::new(&h.encode()[..]));
    }
}
