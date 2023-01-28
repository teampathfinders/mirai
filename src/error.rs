use thiserror::Error;

/// Shorthand for `Result<T, VexError>`.
pub type VexResult<T> = Result<T, VexError>;

/// Verifies that the given expression evaluates to true,
/// or returns an [`AssertionError`](VexError::AssertionError).
#[macro_export]
macro_rules! vex_assert {
    ($expression: expr, $message: expr) => {
        if ($expression) == false {
            return Err($crate::error::VexError::AssertionError($message.into()));
        }
    };

    ($expression: expr) => {
        vex_assert!(
            $expression,
            format!("Assertion failed: {}", stringify!($expression))
        );
    };
}

/// Shorthand used to create error messages
///
/// # Example
/// ```
/// fn fail() -> VexResult<()> {
///     return Err(error!(InvalidRequest, "Received an invalid request!"))
/// }
/// ```
///
#[macro_export]
macro_rules! error {
    ($error_type: ident, $content: expr) => {
        $crate::error::VexError::$error_type($content.into())
    };
}

#[macro_export]
macro_rules! bail {
    ($error_type: ident, $content: expr) => {
        return Err($crate::error!($error_type, $content))
    };
}

/// Custom error type
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum VexError {
    /// An assertion has failed.
    #[error("Assertion failed | {0}")]
    AssertionError(String),
    /// The server received an invalid request.
    #[error("A client sent an invalid request | {0}")]
    InvalidRequest(String),
    /// A synchronisation primitive has failed.
    /// This can happen when a mutex is poisoned due to a panic for example.
    #[error("Synchronisation primitive failed | {0}")]
    SyncPrimitive(String),
    /// An I/O error has occurred.
    /// Any variants of [`io::Error`](std::io::Error) are directly converted to this.
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// Any error that does not fit in the previous categories.
    #[error("{0}")]
    Other(String),
}

impl From<std::num::ParseIntError> for VexError {
    fn from(value: std::num::ParseIntError) -> Self {
        Self::Other(value.to_string())
    }
}

impl<T> From<tokio::sync::SetError<T>> for VexError {
    fn from(value: tokio::sync::SetError<T>) -> Self {
        Self::Other(value.to_string())
    }
}

impl From<serde_json::Error> for VexError {
    fn from(value: serde_json::Error) -> Self {
        Self::InvalidRequest(value.to_string())
    }
}

impl From<base64::DecodeError> for VexError {
    fn from(value: base64::DecodeError) -> Self {
        Self::InvalidRequest(value.to_string())
    }
}

impl From<jsonwebtoken::errors::Error> for VexError {
    fn from(value: jsonwebtoken::errors::Error) -> Self {
        Self::InvalidRequest(value.to_string())
    }
}

impl From<spki::Error> for VexError {
    fn from(value: spki::Error) -> Self {
        Self::InvalidRequest(value.to_string())
    }
}

// impl From<openssl::error::ErrorStack> for VexError {
//     fn from(value: openssl::error::ErrorStack) -> Self {
//         Self::InvalidRequest(value.to_string())
//     }
// }
