#![crate_type="lib"]
#![crate_name="cantal_values"]

extern crate rustc_serialize;
extern crate libc;
#[macro_use] extern crate quick_error;
#[macro_use] extern crate log;
#[macro_use] extern crate probor;
extern crate byteorder;

use std::collections::HashSet;
use std::io::{Cursor, BufReader};
use std::io::{Read, BufRead, Seek};
use std::io::SeekFrom::{Current};
use std::io::Error as IoError;
use std::fs::File;
use std::rc::Rc;
use std::path::Path;
use std::convert::From;
use std::os::unix::fs::MetadataExt;
use rustc_serialize::json;
use rustc_serialize::json::{Json};
use byteorder::{NativeEndian, ReadBytesExt};

use itertools::NextValue;


mod util;
pub mod itertools;


#[derive(Clone, Debug, RustcEncodable, RustcDecodable)]
pub enum Value {
    Counter(u64),
    Integer(i64),
    Float(f64),
    State((u64, String)),
}

probor_enum_encoder_decoder!(Value {
    #0 State(item #1),
    #1 Counter(value #1),
    #2 Integer(value #1),
    #3 Float(value #1),
});

#[derive(Debug, Clone, Copy)]
pub enum LevelType {
    Signed,
    Unsigned,
    Float,
}

#[derive(Debug, Clone, Copy)]
pub enum Type {
    Counter(u8),
    Level(u8, LevelType),
    State(u16),
    Pad(u16),
    Unknown(u16),
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct FileId {
    pub dev: u64,
    pub ino: u64,
}

quick_error! {
    #[derive(Debug)]
    pub enum MetadataError {
        Io(err: IoError) {
            from()
            display("Error reading metadata: {}", err)
            cause(err)
        }
        // TODO(tailhook) add line numbers
        Json(err: json::ParserError) {
            from()
            display("Error parsing metadata json: {}", err)
            cause(err)
        }
        ParseError(desc: &'static str) {
            from()
            display("Error parsing metadata: {}", desc)
        }
        BadLength(len: usize) {
            display("Bad length of a field")
        }
        UnexpectedEOF {
            display("Unexpected end of file")
        }
        Duplicate {
            display("Duplicate values file")
        }
    }
}


#[derive(Debug)]
pub struct Descriptor {
    pub kind: Type,
    pub textname: String,
    pub json: json::Json,
}

pub struct Metadata {
    items: Vec<Rc<Descriptor>>,
    stat: util::Stat,
}

impl<'a> From<&'a ::std::fs::Metadata> for FileId {
    fn from(m: &::std::fs::Metadata) -> FileId {
        FileId {
            dev: (*m).dev(),
            ino: (*m).ino(),
        }
    }
}

impl Value {
    /// Additional bytes on top of size_of_val(self)
    pub fn additional_bytes(&self) -> usize {
        use self::Value::*;
        match self {
            &Counter(_) | &Integer(_) | &Float(_) => 0,
            &State((_, ref v)) => v.as_bytes().len(),
        }
    }
}

impl Metadata {
    pub fn read(path: &Path) -> Result<Metadata, MetadataError> {
        // TODO(tailhook) implement LineNumberReader
        let mut file = BufReader::new(try!(File::open(path)));
        let stat = try!(util::file_stat(file.get_ref()));
        let mut items = vec!();
        loop {
            let mut line = String::new();
            try!(file.read_line(&mut line));
            if line.len() == 0 { break; }
            let mut pair = line.trim()[..].splitn(2, ':');
            let mut type_iter = pair.next().unwrap().split(' ');
            let typ = try!(type_iter.next()
                .ok_or(MetadataError::ParseError("bad type name")));
            let len: usize = try!(type_iter.next_value()
                .map_err(|()| MetadataError::ParseError("bad length")));
            let item = match typ {
                "counter" => {
                    if len > 255 {
                        return Err(MetadataError::BadLength(len));
                    }
                    Type::Counter(len as u8)
                }
                "level" => {
                    if len > 255 {
                        return Err(MetadataError::BadLength(len));
                    }
                    let level_kind = match type_iter.next() {
                        Some("signed") => LevelType::Signed,
                        Some("unsigned") => LevelType::Unsigned,
                        Some("float") => LevelType::Float,
                        _ => return Err(MetadataError::ParseError(
                            "bad kind of \"level\" variable")),
                    };
                    Type::Level(len as u8, level_kind)
                }
                "state" => {
                    if len > 65535 {
                        return Err(MetadataError::BadLength(len));
                    }
                    Type::State(len as u16)
                }
                "pad" => {
                    if len > 65535 {
                        return Err(MetadataError::BadLength(len));
                    }
                    items.push(Rc::new(Descriptor {
                        textname: "".to_string(),
                        json: Json::Null,
                        kind: Type::Pad(len as u16),
                    }));
                    continue;
                }
                _ => {
                    if len > 65535 {
                        return Err(MetadataError::BadLength(len));
                    }
                    Type::Unknown(len as u16)
                }
            };
            let textname = try!(pair.next()
                .ok_or(MetadataError::ParseError("No description for value")));
            let json = try!(Json::from_str(textname));
            items.push(Rc::new(Descriptor {
                textname: textname.trim().to_string(),
                json: json,
                kind: item,
            }));
        }
        return Ok(Metadata {
            items: items,
            stat: stat,
        });
    }
    pub fn read_data(&self, path: &Path, visited: &mut HashSet<FileId>)
        -> Result<Vec<(Rc<Descriptor>, Value)>, MetadataError>
    {
        //  We should read as fast as possible to have more precise results
        //  So we buffer whole file
        // TODO(tailhook) calculate the size of the file when reading metadata
        let mut f = File::open(path)?;
        let ref meta = f.metadata()?;
        let file_id = meta.into();
        if !visited.insert(file_id) {
            return Err(MetadataError::Duplicate);
        }
        let mut buf = Vec::with_capacity(meta.len() as usize + 1);
        f.read_to_end(&mut buf)?;

        let mut stream = Cursor::new(buf);
        let mut res = vec!();
        for desc in self.items.iter() {
            let data = match desc.kind {
                Type::Counter(8) => {
                    Value::Counter(try!(stream.read_u64::<NativeEndian>()))
                }
                Type::Level(8, LevelType::Signed) => {
                    Value::Integer(try!(stream.read_i64::<NativeEndian>()))
                }
                Type::Level(8, LevelType::Float) => {
                    Value::Float(try!(stream.read_f64::<NativeEndian>()))
                }
                Type::State(len) if len > 8 => {
                    let time_ms = try!(stream.read_u64::<NativeEndian>());
                    let pos = stream.position() as usize;
                    let end = pos+(len as usize)-8;
                    let text = {
                        let buf = stream.get_ref();
                        if buf.len() < end {
                            return Err(MetadataError::UnexpectedEOF);
                        }
                        let slice = &buf[pos..end];
                        if let Some(end) = slice.iter().position(|&x| x == 0) {
                            String::from_utf8_lossy(&slice[0..end]).to_string()
                        } else {
                            String::from_utf8_lossy(&slice[..]).to_string()
                        }
                    };
                    try!(stream.seek(Current((len-8) as i64)));
                    Value::State((time_ms, text))
                }
                Type::Pad(x) => {
                    try!(stream.seek(Current(x as i64)));
                    continue;
                }
                x => {
                    warn!("Type {:?} cannot be read", x);
                    try!(stream.seek(Current(x.len() as i64)));
                    continue;
                }
            };
            res.push((desc.clone(), data));
        }
        return Ok(res);
    }
    pub fn still_fresh(&self, path: &Path) -> bool {
        let stat = util::path_stat(path);
        return Some(&self.stat) == stat.as_ref().ok();
    }
}

impl Type {
    fn len(&self) -> usize {
        match *self {
            Type::Counter(len) => len as usize,
            Type::Level(len, _) => len as usize,
            Type::State(len) => len as usize,
            Type::Pad(len) => len as usize,
            Type::Unknown(len) => len as usize,
        }
    }
}
