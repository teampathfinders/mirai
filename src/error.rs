use thiserror::Error;

pub type VexResult<T> = Result<T, VexError>;

#[macro_export]
macro_rules! vex_assert {
    ($expression: expr, $message: expr) => {
        if ($expression) == false {
            return Err($crate::error::VexError::Assertion($message));
        }
    };

    ($expression: expr) => {
        vex_assert!(
            $expression,
            format!("Assertion failed: {}", stringify!($expression))
        );
    };
}

#[derive(Debug, Error)]
pub enum VexError {
    #[error("Non-fatal assertion failed | {0}")]
    Assertion(String),
    #[error("A client sent an invalid request | {0}")]
    InvalidRequest(String),
    #[error("Synchronisation primitive failed | {0}")]
    SyncPrimitive(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("Unknown error")]
    Other,
}

impl<T> From<std::sync::PoisonError<T>> for VexError {
    fn from(error: std::sync::PoisonError<T>) -> VexError {
        VexError::SyncPrimitive(error.to_string())
    }
}
