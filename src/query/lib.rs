#![crate_type="lib"]
#![crate_name="cantal"]

extern crate serialize;
extern crate libc;
#[macro_use] extern crate log;

use std::str::FromStr;
use std::fmt::String as FmtString;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use std::io::{Cursor, BufReader};
use std::io::{Read, BufRead};
use std::io::Error as IoError;
use std::fs::File;
use std::rc::Rc;
use std::error::{Error, FromError};
use serialize::json;
use serialize::json::Json;

use itertools::NextValue;
use iotools::ReadHostBytes;


mod util;
pub mod itertools;
pub mod iotools;


#[derive(Show)]
pub enum Value {
    Counter(u64),
    Integer(i64),
    Float(f64),
    State(u64, String),
}

#[derive(Show, Copy)]
pub enum LevelType {
    Signed,
    Unsigned,
    Float,
}

#[derive(Show, Copy)]
pub enum Type {
    Counter(u8),
    Level(u8, LevelType),
    State(u16),
    Pad(u8),
    Unknown(u16),
}

#[derive(Show)]
pub enum MetadataError {
    Io(IoError),
    // TODO(tailhook) add line numbers
    Json(json::ParserError),
    ParseError(&'static str),
    BadLength(usize),
}


pub struct Descriptor {
    pub kind: Type,
    pub textname: String,
    pub json: json::Json,
}

pub struct Metadata {
    items: Vec<Rc<Descriptor>>,
    stat: util::Stat,
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
            let mut pair = line.trim().as_slice().splitn(1, ':');
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
                    if len > 255 {
                        return Err(MetadataError::BadLength(len));
                    }
                    items.push(Rc::new(Descriptor {
                        textname: "".to_string(),
                        json: Json::Null,
                        kind: Type::Pad(len as u8),
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
            let json = try!(json::from_str(textname));
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
    pub fn read_data(&self, path: &Path)
        -> Result<Vec<(Rc<Descriptor>, Value)>, MetadataError>
    {
        //  We should read as fast as possible to have more precise results
        //  So we buffer whole file
        let mut buf = Vec::with_capacity(4096);
        try!(File::open(path)
            .and_then(|mut f| f.read_to_end(&mut buf)));

        let mut stream = BufReader::new(Cursor::new(buf));
        let mut res = vec!();
        for desc in self.items.iter() {
            let data = match desc.kind {
                Type::Counter(8) => {
                    Value::Counter(try!(stream.read_u64()))
                }
                Type::Level(8, LevelType::Signed) => {
                    Value::Integer(try!(stream.read_i64()))
                }
                Type::Level(8, LevelType::Float) => {
                    Value::Float(try!(stream.read_f64()))
                }
                Type::State(len) if len > 8 => {
                    let time_ms = try!(stream.read_u64());
                    let val = try!(stream.read_bytes((len - 8) as usize));
                    let text = if let Some(end) =
                        val.as_slice().position_elem(&0)
                    {
                        String::from_utf8_lossy(&val.as_slice()[0..end])
                    } else {
                        String::from_utf8_lossy(val.as_slice())
                    };
                    Value::State(time_ms, text.to_string())
                }
                Type::Pad(x) => {
                    try!(stream.read_bytes(x as usize));
                    continue;
                }
                x => {
                    warn!("Type {:?} cannot be read", x);
                    try!(stream.read_bytes(x.len()));
                    continue;
                }
            };
            res.push((desc.clone(), data));
        }
        return Ok(res);
    }
}

impl Error for MetadataError {
    fn description(&self) -> &str {
        match *self {
            MetadataError::Io(ref err) => err.description(),
            MetadataError::ParseError(desc)
            => "Error parsing metadata file",
            MetadataError::BadLength(_)
            => "Error parsing metadata file: wrong field length",
            MetadataError::Json(_)
            => "Error parsing metadata file: bad json",
        }
    }
    fn cause<'x>(&'x self) -> Option<&'x Error> {
        match *self {
            MetadataError::Io(ref err) => Some(err as &Error),
            MetadataError::Json(ref err) => None,  // json::ParserError sucks
            MetadataError::ParseError(_) | MetadataError::BadLength(_) => None,
        }
    }
}

impl FmtString for MetadataError {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        match *self {
            MetadataError::Io(ref err) => {
                write!(fmt, "metadata file read error: {}", err)
            }
            MetadataError::Json(ref err) => {
                write!(fmt, "metadata file read error: {:?}", err)
            }
            MetadataError::ParseError(desc) => {
                write!(fmt, "error parsing metadata file: {}", desc)
            }
            MetadataError::BadLength(val) => {
                write!(fmt, "error parsing metadata file: field length {} \
                    is not supported", val)
            }
        }
    }
}

impl FromError<IoError> for MetadataError {
    fn from_error(err: IoError) -> MetadataError {
        MetadataError::Io(err)
    }
}

impl FromError<json::ParserError> for MetadataError {
    fn from_error(err: json::ParserError) -> MetadataError {
        MetadataError::Json(err)
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
