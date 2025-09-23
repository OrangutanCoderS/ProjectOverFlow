use thiserror::Error;

#[derive(Debug, Error)]
pub enum InjectorError {
    #[error("global injector not initialized")]
    NotInitialized,
}

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("invalid PID: {0}")]
    BadPid(i32),
    #[error("PID is protected by policy: {0}")]
    ProtectedPid(i32),
}

#[derive(Debug, Error)]
pub enum ExecutionError {
    #[error("handler unavailable for action")]
    NoHandler,
    #[error("handler failed: {0}")]
    HandlerFailure(String),
}
