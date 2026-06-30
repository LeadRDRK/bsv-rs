use num_enum::TryFromPrimitive;

use crate::{Error, ext::ReadExt};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, TryFromPrimitive)]
#[repr(u8)]
pub enum ValueType {
    UShort = 16,
    UInt,
    ULong,
    Unknown19 = 19, // probably padding?
    UNum = 32, // this thing is an oxymoron
    UNumFixed, // <- because of that
    Blob = 48,
    BlobFixed,
    Text = 64,
    TextFixed
}

impl ValueType {
    pub fn is_fixed(&self) -> bool {
        match self {
            Self::UNumFixed | Self::BlobFixed | Self::TextFixed => true,
            _ => false
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Value {
    UShort(u16),
    UInt(u32),
    ULong(u64),
    Unknown19,
    UNum(u64),
    UNumFixed(u64),
    Blob(Vec<u8>),
    BlobFixed(Vec<u8>),
    Text(String),
    TextFixed(String)
}

impl Value {
    pub fn u16(&self) -> Option<u16> {
        match self {
            Self::UShort(v) => Some(*v),
            Self::UNum(v) | Self::UNumFixed(v) => (*v).try_into().ok(),
            _ => None
        }
    }

    pub fn u32(&self) -> Option<u32> {
        match self {
            Self::UInt(v) => Some(*v),
            Self::UNum(v) | Self::UNumFixed(v) => (*v).try_into().ok(),
            _ => None
        }
    }

    pub fn u64(&self) -> Option<u64> {
        match self {
            Self::ULong(v) | Self::UNum(v) | Self::UNumFixed(v) => Some(*v),
            _ => None
        }
    }

    pub fn unum(&self) -> Option<u64> {
        match self {
            Self::UShort(v) => Some(*v as _),
            Self::UInt(v) => Some(*v as _),
            Self::ULong(v) | Self::UNum(v) | Self::UNumFixed(v) => Some(*v),
            _ => None
        }
    }

    pub fn blob(&self) -> Option<&[u8]> {
        match self {
            Self::Blob(b) | Self::BlobFixed(b) => Some(b),
            _ => None
        }
    }

    pub fn text(&self) -> Option<&str> {
        match self {
            Self::Text(s) | Self::TextFixed(s) => Some(s),
            _ => None
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Schema {
    pub value_type: ValueType,
    pub fixed_size: Option<u64>
}

impl Schema {
    pub fn parse<R: ReadExt>(mut reader: R) -> Result<Self, Error> {
        let value_type = ValueType::try_from_primitive(reader.read_u8()?)
            .map_err(|e| Error::InvalidSchemaType(e.number))?;
        let fixed_size = value_type.is_fixed()
            .then(|| reader.read_vlq(8))
            .transpose()?;

        Ok(Self {
            value_type,
            fixed_size
        })
    }

    pub fn read_value<R: ReadExt>(&self, mut reader: R) -> Result<Value, Error> {
        Ok(match (self.value_type, self.fixed_size) {
            (ValueType::UShort, _) => Value::UShort(reader.read_vlq(2)? as _),
            (ValueType::UInt, _) => Value::UInt(reader.read_vlq(4)? as _),
            (ValueType::ULong, _) => Value::ULong(reader.read_vlq(8)? as _),
            (ValueType::UNum, _) => return Err(Error::Unimplemented("reading non-fixed UNum value")), // oxymoron

            (ValueType::UNumFixed, Some(fixed_size)) => reader.read_unum(fixed_size as _)
                .map(Value::UNumFixed)?,

            (ValueType::Blob, _) => reader.read_vlq(8)
                .and_then(|len| reader.read_blob(len as _))
                .map(Value::Blob)?,

            (ValueType::BlobFixed, Some(fixed_size)) => reader.read_blob(fixed_size as _)
                .map(Value::BlobFixed)?,

            (ValueType::Text, _) => reader.read_null_terminated_string()
                .map(Value::Text)?,

            (ValueType::TextFixed, Some(fixed_size)) => reader.read_blob(fixed_size as _)
                .map(|buf| String::from_utf8(buf))?
                .map(Value::TextFixed)?,

            (ValueType::Unknown19, _) => reader.read_exact(&mut [0u8; 8])
                .map(|_| Value::Unknown19)?,

            _ => return Err(Error::InvalidSchema)
        })
    }
}