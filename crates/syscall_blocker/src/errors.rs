use thiserror::Error;

/// Unified error types for syscall_blocker.
#[derive(Debug, Error)]
pub enum SysBlockError {
    #[error("Invalid PID: {0}")]
    InvalidPid(String),

    #[error("Syscall not permitted: {0}")]
    SyscallDenied(String),

    #[error("I/O or system error: {0}")]
    IoError(String),

    #[error("Permission denied (requires root privileges)")]
    PermissionDenied,

    #[error("Platform not supported for syscall interception")]
    UnsupportedPlatform,

    #[error("Internal logic error: {0}")]
    Internal(String),
}