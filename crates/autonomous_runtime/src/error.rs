use thiserror::Error;

/// Errors that can occur inside the autonomous runtime.
#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("invalid runtime config: {0}")]
    InvalidConfig(String),

    #[error("event source error: {0}")]
    EventSource(String),

    #[error("action sink error: {0}")]
    ActionSink(String),

    #[error("policy evaluation error: {0}")]
    Policy(String),

    #[error("threshold evaluation error: {0}")]
    Threshold(String),

    #[error("runtime is not running (status: {0:?})")]
    NotRunning(super::state::RuntimeStatus),

    #[error("runtime is paused")]
    Paused,

    #[error("internal runtime error: {0}")]
    Internal(String),
}

impl RuntimeError {
    /// Convert any error-like value into a RuntimeError::Internal.
    pub fn internal<E: std::fmt::Display>(err: E) -> Self {
        RuntimeError::Internal(err.to_string())
    }
}