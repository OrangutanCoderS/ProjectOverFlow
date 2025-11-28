use thiserror::Error;
use std::io;

#[derive(Debug, Error)]
pub enum PatchError {
    #[error("validation error: {0}")]
    Validation(String),

    #[error("permission denied: {0}")]
    PermissionDenied(String),

    #[error("target process not found or exited")]
    ProcessGone,

    #[error("region/signature not found")]
    NotFound,

    #[error("checksum mismatch")]
    ChecksumMismatch,

    #[error("operation timed out")]
    Timeout,

    #[error("rollback failed: {0}")]
    RollbackFailed(String),

    #[error("unsupported platform/feature disabled")]
    Unsupported,

    #[error("OS error: {0}")]
    Os(#[from] io::Error),

    #[error("internal error: {0}")]
    Internal(String),
}