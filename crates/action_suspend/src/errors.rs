use thiserror::Error;

#[derive(Error, Debug)]
pub enum ManagerError {
    #[error("Invalid PID: {0}")]
    InvalidPid(i32),

    #[error("Permission denied for PID: {0}")]
    PermissionDenied(i32),

    #[error("Failed to send signal to PID {0}: {1}")]
    SignalFailed(i32, String),

    #[error("Audit log write failed: {0}")]
    AuditWriteError(String),
}