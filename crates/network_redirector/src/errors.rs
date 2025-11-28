use thiserror::Error;

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("invalid match type/value: {0}")]
    InvalidMatch(String),

    #[error("redirect target missing for redirect mode")]
    MissingRedirectTarget,

    #[error("PID {0} is protected")]
    PidProtected(i32),

    #[error("request denied by security policy: {0}")]
    Denylisted(String),

    #[error("ttl too large: {0}")]
    TtlTooLarge(u64),
}

#[derive(Debug, Error)]
pub enum BackendError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("permission denied")]
    PermissionDenied,

    #[error("operation not supported")]
    NotSupported,

    #[error("rule not found")]
    NotFound,

    #[error("conflict: {0}")]
    Conflict(String),

    #[error("parse error: {0}")]
    ParseError(String),
}

#[derive(Debug, Error)]
pub enum ManagerError {
    #[error("validation error: {0}")]
    Validation(#[from] ValidationError),

    #[error("backend error: {0}")]
    Backend(#[from] BackendError),

    #[error("logging error: {0}")]
    Logging(String),

    #[error("internal error: {0}")]
    Internal(String),
}