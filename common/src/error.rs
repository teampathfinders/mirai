use std::backtrace::Backtrace;

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

#[macro_export]
macro_rules! bail {
    ($err_type: ident, $fmt: expr, $($args:expr),+) => {
        return Err($crate::VError::new($crate::VErrorKind::$err_type, format!($fmt, $($args),+)))
    };

    ($err_type: ident, $fmt: expr) => {
        return Err($crate::VError::new($crate::VErrorKind::$err_type, format!($fmt)))
    };

    ($err_type: ident) => {
        $crate::bail!($err_type, "No description")
    };
}

#[macro_export]
macro_rules! error {
    ($err_type: ident, $fmt: expr, $($args:expr),+) => {
        $crate::VError::new($crate::VErrorKind::$err_type, format!($fmt, $($args),+))
    };

    ($err_type: ident, $fmt: expr) => {
        $crate::VError::new($crate::VErrorKind::$err_type, $fmt.to_string())
    };
}

pub type VResult<T> = Result<T, VError>;

#[derive(Debug, Copy, Clone)]
pub enum VErrorKind {
    /// An assertion failed.
    AssertionFailure,
    /// Something has been aborted.
    Aborted,
    /// Client is not authenticated.
    NotAuthenticated,
    /// Something was not connected yet.
    NotConnected,
    /// Client sent a bad packet.
    BadPacket,
    /// Version mismatch.
    VersionMismatch,
    /// The server tried to initialise something that was already initialised.
    AlreadyInitialized,
    /// The server tried to use something that hasn't been initialised yet.
    NotInitialized,
    /// Something is already in use.
    AlreadyInUse,
    /// Client sent invalid identity keys during login.
    InvalidIdentity,
    /// An operation on the database has failed.
    DatabaseFailure,
    /// An invalid chunk was found.
    InvalidChunk,
    /// An invalid NBT structure was encountered.
    InvalidNbt,
    /// An invalid skin was given.
    InvalidSkin,
    /// The client sent an invalid command.
    InvalidCommand,
    /// An unknown error
    Other,
}

#[derive(Debug)]
pub struct VError {
    kind: VErrorKind,
    backtrace: Backtrace,
    message: String,
}

impl VError {
    #[inline]
    pub fn new(kind: VErrorKind, message: String) -> Self {
        Self {
            kind,
            message,
            backtrace: Backtrace::capture(),
        }
    }

    #[inline]
    pub const fn kind(&self) -> VErrorKind {
        self.kind
    }

    #[inline]
    pub const fn backtrace(&self) -> &Backtrace {
        &self.backtrace
    }
}

impl std::error::Error for VError {}

impl std::fmt::Display for VError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "{:?} | {}", self.kind, self.message)
    }
}

impl<T> From<tokio::sync::SetError<T>> for VError {
    fn from(value: tokio::sync::SetError<T>) -> Self {
        Self::new(VErrorKind::AlreadyInitialized, value.to_string())
    }
}

impl From<std::io::Error> for VError {
    fn from(value: std::io::Error) -> Self {
        match value.kind() {
            std::io::ErrorKind::InvalidData => {
                Self::new(VErrorKind::BadPacket, value.to_string())
            }
            std::io::ErrorKind::AlreadyExists => {
                Self::new(VErrorKind::AlreadyInitialized, value.to_string())
            }
            std::io::ErrorKind::AddrInUse => {
                Self::new(VErrorKind::AlreadyInUse, value.to_string())
            }
            std::io::ErrorKind::NotConnected => {
                Self::new(VErrorKind::NotConnected, value.to_string())
            }
            std::io::ErrorKind::ConnectionAborted => {
                Self::new(VErrorKind::Aborted, value.to_string())
            }
            _ => Self::new(VErrorKind::Other, value.to_string()),
        }
    }
}

impl From<jsonwebtoken::errors::Error> for VError {
    fn from(value: jsonwebtoken::errors::Error) -> Self {
        match value.kind() {
            jsonwebtoken::errors::ErrorKind::InvalidToken => {
                Self::new(VErrorKind::BadPacket, value.to_string())
            }
            jsonwebtoken::errors::ErrorKind::InvalidSignature => {
                Self::new(VErrorKind::InvalidIdentity, value.to_string())
            }
            jsonwebtoken::errors::ErrorKind::InvalidEcdsaKey => {
                Self::new(VErrorKind::InvalidIdentity, value.to_string())
            }
            jsonwebtoken::errors::ErrorKind::Base64(_)
            | jsonwebtoken::errors::ErrorKind::Json(_)
            | jsonwebtoken::errors::ErrorKind::Utf8(_) => {
                Self::new(VErrorKind::BadPacket, value.to_string())
            }
            _ => Self::new(VErrorKind::Other, value.to_string()),
        }
    }
}

impl From<base64::DecodeError> for VError {
    fn from(value: base64::DecodeError) -> Self {
        Self::new(VErrorKind::BadPacket, value.to_string())
    }
}

impl From<serde_json::Error> for VError {
    fn from(value: serde_json::Error) -> Self {
        Self::new(VErrorKind::BadPacket, value.to_string())
    }
}

impl From<std::num::ParseIntError> for VError {
    fn from(value: std::num::ParseIntError) -> Self {
        Self::new(VErrorKind::BadPacket, value.to_string())
    }
}

impl From<std::ffi::NulError> for VError {
    fn from(value: std::ffi::NulError) -> Self {
        Self::new(VErrorKind::DatabaseFailure, value.to_string())
    }
}

impl<T> From<snap::write::IntoInnerError<T>> for VError {
    fn from(value: snap::write::IntoInnerError<T>) -> Self {
        Self::new(VErrorKind::Other, value.to_string())
    }
}

impl From<dashmap::TryReserveError> for VError {
    fn from(_: dashmap::TryReserveError) -> Self {
        Self::new(VErrorKind::Other, "Failed to reserve Dashmap space".to_owned())
    }
}