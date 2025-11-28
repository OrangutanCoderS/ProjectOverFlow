use thiserror::Error;

#[derive(Debug, Error)]
pub enum ManagerError {
    #[error("Invalid PID: {0}")]
    InvalidPid(i32),
    #[error("Operation failed: {0}")]
    OperationFailed(String),
    #[error("Unsupported operation on this platform")]
    UnsupportedPlatform,
    #[error("Audit log error: {0}")]
    Audit(String),
}