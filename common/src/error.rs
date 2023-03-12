/// Verifies that the given expression evaluates to true,
/// or returns an error
#[macro_export]
macro_rules! nvassert {
    ($expression: expr, $message: expr) => {
        if ($expression) == false {
            $crate::bail!(AssertionFailure, "{} | {}", $expression, $message);
        }
    };

    ($expression: expr) => {
        nvassert!(
            $expression,
            format!("Assertion failed: {}", stringify!($expression))
        );
    };
}

/// Bails from a function early, returning the specified error.
#[macro_export]
macro_rules! bail {
    ($err_type: ident, $fmt: expr, $($args:expr),+) => {
        return Err($crate::Error::new($crate::ErrorKind::$err_type, format!($fmt, $($args),+)))
    };

    ($err_type: ident, $fmt: expr) => {
        return Err($crate::Error::new($crate::ErrorKind::$err_type, format!($fmt)))
    };

    ($err_type: ident) => {
        $crate::bail!($err_type, "No description")
    };
}

/// Creates a new [`Error`].
///
#[macro_export]
macro_rules! error {
    ($err_type: ident, $fmt: expr, $($args:expr),+) => {
        $crate::Error::new($crate::ErrorKind::$err_type, format!($fmt, $($args),+))
    };

    ($err_type: ident, $fmt: expr) => {
        $crate::Error::new($crate::ErrorKind::$err_type, $fmt.to_string())
    };
}

/// Shorthand for `Result<T, Error>`.
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Copy, Clone)]
pub enum ErrorKind {
    UnexpectedEof,
    /// An assertion failed.
    AssertionFailed,
    /// Client is not authenticated.
    NotAuthenticated,
    /// Client sent a bad packet.
    Malformed,
    /// Version mismatch.
    Outdated,
    /// The server tried to initialise something that was already initialised.
    AlreadyInitialized,
    /// The server tried to use something that hasn't been initialised yet.
    NotInitialized,
    /// An operation on the database has failed.
    DatabaseFailure,
    /// An unknown error
    Other,
}

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    message: String,
}

impl Error {
    #[inline]
    pub fn new(kind: ErrorKind, message: String) -> Self {
        Self { kind, message }
    }

    #[inline]
    pub const fn kind(&self) -> ErrorKind {
        self.kind
    }
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "{:?} | {}", self.kind, self.message)
    }
}

impl<T> From<tokio::sync::SetError<T>> for Error {
    fn from(value: tokio::sync::SetError<T>) -> Self {
        Self::new(ErrorKind::AlreadyInitialized, value.to_string())
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        match value.kind() {
            std::io::ErrorKind::InvalidData => {
                Self::new(ErrorKind::Malformed, value.to_string())
            }
            std::io::ErrorKind::AlreadyExists => {
                Self::new(ErrorKind::AlreadyInitialized, value.to_string())
            }
            std::io::ErrorKind::NotConnected => {
                Self::new(ErrorKind::NotAuthenticated, value.to_string())
            }
            _ => Self::new(ErrorKind::Other, value.to_string()),
        }
    }
}

impl From<jsonwebtoken::errors::Error> for Error {
    fn from(value: jsonwebtoken::errors::Error) -> Self {
        match value.kind() {
            jsonwebtoken::errors::ErrorKind::InvalidToken
            | jsonwebtoken::errors::ErrorKind::InvalidSignature
            | jsonwebtoken::errors::ErrorKind::InvalidEcdsaKey
            | jsonwebtoken::errors::ErrorKind::Base64(_)
            | jsonwebtoken::errors::ErrorKind::Json(_)
            | jsonwebtoken::errors::ErrorKind::Utf8(_) => {
                Self::new(ErrorKind::Malformed, value.to_string())
            }
            _ => Self::new(ErrorKind::Other, value.to_string()),
        }
    }
}

impl From<base64::DecodeError> for Error {
    fn from(value: base64::DecodeError) -> Self {
        Self::new(ErrorKind::Malformed, value.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Self::new(ErrorKind::Malformed, value.to_string())
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(value: std::num::ParseIntError) -> Self {
        Self::new(ErrorKind::Malformed, value.to_string())
    }
}

impl From<std::ffi::NulError> for Error {
    fn from(value: std::ffi::NulError) -> Self {
        Self::new(ErrorKind::DatabaseFailure, value.to_string())
    }
}

impl<T> From<snap::write::IntoInnerError<T>> for Error {
    fn from(value: snap::write::IntoInnerError<T>) -> Self {
        Self::new(ErrorKind::Other, value.to_string())
    }
}

impl From<dashmap::TryReserveError> for Error {
    fn from(_: dashmap::TryReserveError) -> Self {
        Self::new(
            ErrorKind::Other,
            "Failed to reserve Dashmap space".to_owned(),
        )
    }
}

impl<T> From<tokio::sync::mpsc::error::SendError<T>> for Error {
    fn from(value: tokio::sync::mpsc::error::SendError<T>) -> Self {
        Self::new(ErrorKind::Other, value.to_string())
    }
}

impl<T> From<tokio::sync::broadcast::error::SendError<T>> for Error {
    fn from(value: tokio::sync::broadcast::error::SendError<T>) -> Self {
        Self::new(ErrorKind::Other, value.to_string())
    }
}

impl From<cipher::StreamCipherError> for Error {
    fn from(value: cipher::StreamCipherError) -> Self {
        Self::new(ErrorKind::Malformed, value.to_string())
    }
}
