use thiserror::Error;

pub type VexResult<T> = Result<T, VexError>;

#[derive(Debug, Error)]
pub enum VexError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error("Unknown error")]
    Other,
}
