use std::fmt::Display;

#[derive(Debug)]
pub enum Error {
    InvalidMagic,
    Unimplemented(&'static str),
    IoError(std::io::Error),
    InvalidSchema,
    InvalidSchemaType(u8),
    FromUtf8Error(std::string::FromUtf8Error),
    BrokenReader
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidMagic => f.write_str("Invalid magic number"),
            Self::Unimplemented(reason) => write!(f, "Unimplemented: {}", reason),
            Self::IoError(e) => e.fmt(f),
            Self::InvalidSchema => f.write_str("Invalid schema"),
            Self::InvalidSchemaType(typ) => write!(f, "Invalid schema type: {}", typ),
            Self::FromUtf8Error(e) => e.fmt(f),
            Self::BrokenReader => f.write_str("Reader encountered an error and cannot continue")
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::IoError(value)
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(value: std::string::FromUtf8Error) -> Self {
        Self::FromUtf8Error(value)
    }
}