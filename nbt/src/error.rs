use serde::{de, ser};
use std::fmt::Display;
use std::{fmt, io};

#[macro_export]
macro_rules! bail {
    ($x: ident) => {
        return Err($crate::error::Error::$x)
    };
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    InvalidUtf8,
    Unsupported,
    UnexpectedEof,
    /// An invalid tag type was encountered.
    InvalidType,
    /// There are unread bytes remaining in the given buffer.
    TrailingBytes,
    /// A custom error message.
    Custom(String),
}

impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Custom(msg.to_string())
    }
}

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Custom(msg.to_string())
    }
}

impl Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Custom(msg) => formatter.write_str(msg),
            _ => todo!(),
        }
    }
}

impl std::error::Error for Error {}

impl From<io::Error> for Error {
    fn from(v: io::Error) -> Self {
        match v.kind() {
            io::ErrorKind::UnexpectedEof => Self::UnexpectedEof,
            _ => Self::Custom(v.to_string()),
        }
    }
}

impl From<std::str::Utf8Error> for Error {
    fn from(_v: std::str::Utf8Error) -> Self {
        Error::InvalidUtf8
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(_v: std::string::FromUtf8Error) -> Self {
        Error::InvalidUtf8
    }
}
