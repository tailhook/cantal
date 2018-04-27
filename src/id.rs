use std::fmt;
use std::str::FromStr;
use std::sync::Arc;

use hex::{FromHex, ToHex, encode as hexlify, FromHexError};
use serde::{Serialize, Serializer};
use serde::de::{Deserialize, Deserializer, Error as DeError, Visitor};


#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Id(InternalId);

struct IdVisitor;

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum InternalId {
    Good([u8; 16]),
    Bad(Arc<[u8]>),
}

impl Id {
    pub fn new<S:AsRef<[u8]>>(id: S) -> Id {
        let id = id.as_ref();
        if id.len() == 16 {
            let mut x = [0u8; 16];
            x.copy_from_slice(id);
            Id(InternalId::Good(x))
        } else {
            Id(InternalId::Bad(id.to_vec().into()))
        }
    }
    pub fn to_hex(&self) -> String {
        match self.0 {
            InternalId::Good(ar) => hexlify(ar),
            InternalId::Bad(ref vec) => hexlify(&vec[..]),
        }
    }
}

impl Serialize for Id {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        self.to_hex().serialize(serializer)
    }
}

impl FromStr for Id {
    type Err = FromHexError;
    fn from_str(s: &str) -> Result<Id, Self::Err> {
        let ar: Vec<u8> = FromHex::from_hex(s.as_bytes())?;
        if ar.len() == 16 {
            Ok(Id::new(ar))
        } else {
            Ok(Id(InternalId::Bad(ar.into())))
        }
    }
}

impl fmt::Display for Id {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self.0 {
            InternalId::Good(ar) => {
                ar.write_hex(fmt)
            }
            InternalId::Bad(ref vec) => {
                vec.write_hex(fmt)
            }
        }
    }
}

impl fmt::Debug for Id {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self.0 {
            InternalId::Good(ar) => {
                write!(fmt, "Id({})", hexlify(ar))
            }
            InternalId::Bad(ref vec) => {
                write!(fmt, "Id({})", hexlify(&vec[..]))
            }
        }
    }
}

impl<'a> Visitor<'a> for IdVisitor {
    type Value = Id;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("bytes or str")
    }
    fn visit_bytes<E>(self, value: &[u8]) -> Result<Self::Value, E>
        where E: DeError
    {
        if value.len() == 16 {
            let mut array = [0u8; 16];
            array.copy_from_slice(value);
            Ok(Id(InternalId::Good(array)))
        } else {
            Ok(Id(InternalId::Bad(value.to_vec().into())))
        }
    }
    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where E: DeError
    {
        Id::from_str(value).map_err(|e| E::custom(e))
    }
}

impl<'a> Deserialize<'a> for Id {
    fn deserialize<D>(deserializer: D) -> Result<Id, D::Error>
        where D: Deserializer<'a>
    {
        deserializer.deserialize_bytes(IdVisitor)
    }
}
