use thiserror::Error;

/// Shorthand for `Result<T, VexError>`.
pub type VexResult<T> = Result<T, VexError>;

/// Verifies that the given expression evaluates to true,
/// or returns an [`AssertionError`](VexError::AssertionError).
#[macro_export]
macro_rules! vex_assert {
    ($expression: expr, $message: expr) => {
        if ($expression) == false {
            return Err($crate::error::VexError::AssertionError($message));
        }
    };

    ($expression: expr) => {
        vex_assert!(
            $expression,
            format!("Assertion failed: {}", stringify!($expression))
        );
    };
}

#[macro_export]
macro_rules! vex_error {
    ($error_type: ident, $content: expr) => {
        $crate::error::VexError::$error_type($content.to_string())
    };
}

#[derive(Debug, Error)]
pub enum VexError {
    /// An assertion has failed.
    #[error("Non-fatal assertion failed | {0}")]
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
    #[error("Unknown error")]
    Other,
}

/// Allow converting poison errors to [`VexError`].
impl<T> From<std::sync::PoisonError<T>> for VexError {
    fn from(error: std::sync::PoisonError<T>) -> VexError {
        VexError::SyncPrimitive(error.to_string())
    }
}
