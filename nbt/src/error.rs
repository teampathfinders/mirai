use std::fmt;
use std::fmt::Display;
use serde::{de, ser};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    Custom(String)
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
            _ => todo!()
        }
    }
}

impl std::error::Error for Error {}