use thiserror::Error;

#[derive(Debug, Error)]
pub enum ManagerError {
    #[error("validation error: {0}")]
    Validation(String),
    #[error("spawn error: {0}")]
    Spawn(String),
    #[error("audit error: {0}")]
    Audit(String),
    #[error("unsupported feature: {0}")]
    Unsupported(String),
}

#[derive(Debug, Error)]
pub enum ControllerError {
    #[error("spawn failed: {0}")]
    Spawn(String),
}