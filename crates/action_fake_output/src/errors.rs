use thiserror::Error;

#[derive(Debug, Error)]
pub enum FakeOutputError {
    #[error("I/O redirection failed: {0}")]
    Io(String),

    #[error("Process {0} not found or inaccessible")]
    ProcessUnavailable(i32),

    #[error("File path invalid or restricted: {0}")]
    InvalidPath(String),

    #[error("JSON mapping or data parse error: {0}")]
    Parse(String),

    #[error("Permission denied during redirection or simulation")]
    PermissionDenied,

    #[error("Operation not supported on current OS")]
    UnsupportedPlatform,

    #[error("Unknown internal error: {0}")]
    Unknown(String),
}