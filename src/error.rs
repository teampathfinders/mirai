use thiserror::Error;

pub type VexResult<T> = Result<T, VexError>;

#[macro_export]
macro_rules! vex_check {
    ($expression: expr, $message: expr) => {
        if ($expression) == false {
            return Err(
                $crate::error::VexError::AssertionFailed($message)
            )
        }
    };

    ($expression: expr) => {
        vex_check!($expression, format!("Assertion failed: {}", stringify!($expression)));
    }
}

#[derive(Debug, Error)]
pub enum VexError {
    #[error("Non-fatal assertion failed: {0}")]
    AssertionFailed(String),

    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error("Unknown error")]
    Other,
}
