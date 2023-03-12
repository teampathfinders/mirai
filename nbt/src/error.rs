use serde::{de, ser};
use std::backtrace::Backtrace;
use std::fmt::{Debug, Display};
use std::{fmt, io};

#[macro_export]
macro_rules! bail {
    ($k: ident, $s: expr, $($arg: tt)*) => {
        return Err($crate::error::Error::new(
            $crate::error::ErrorKind::$k,
            format!($s, $($arg)*)
        ))
    };

    ($k: ident, $s: expr) => {
        return Err($crate::error::Error::new(
            $crate::error::ErrorKind::$k,
            format!($s)
        ))
    };

    ($x: ident) => {
        return Err($crate::error::Error::new($crate::error::ErrorKind::$x, "unknown error"))
    };
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct Error {
    backtrace: Backtrace,
    kind: ErrorKind,
    msg: String,
}

impl Error {
    pub fn new<S: Into<String>>(kind: ErrorKind, msg: S) -> Self {
        Self {
            backtrace: Backtrace::capture(),
            kind,
            msg: msg.into(),
        }
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum ErrorKind {
    InvalidUtf8,
    Unsupported,
    UnexpectedEof,
    /// An invalid tag type was encountered.
    InvalidType,
    /// There are unread bytes remaining in the given buffer.
    TrailingBytes,
    /// A custom error message.
    Other,
}

impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::new(ErrorKind::Other, msg.to_string())
    }
}

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::new(ErrorKind::Other, msg.to_string())
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter
            .debug_struct("Error")
            .field("kind", &self.kind)
            .field("msg", &self.msg)
            .finish()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "{} ({:?})\nbacktrace:\n{}",
            self.msg, self.kind, self.backtrace
        )
    }
}

impl std::error::Error for Error {}

impl From<io::Error> for Error {
    fn from(v: io::Error) -> Self {
        match v.kind() {
            io::ErrorKind::UnexpectedEof => {
                Self::new(ErrorKind::UnexpectedEof, v.to_string())
            }
            _ => Self::new(ErrorKind::Other, v.to_string()),
        }
    }
}

impl From<std::str::Utf8Error> for Error {
    fn from(v: std::str::Utf8Error) -> Self {
        Error::new(ErrorKind::InvalidUtf8, v.to_string())
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(v: std::string::FromUtf8Error) -> Self {
        Error::new(ErrorKind::InvalidUtf8, v.to_string())
    }
}
