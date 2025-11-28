use thiserror::Error;
use std::io;

#[derive(Debug, Error)]
pub enum ManagerError {
    #[error("invalid configuration: {0}")]
    Config(String),

    #[error("policy not matched: {0}")]
    Policy(String),

    #[error("permission denied: {0}")]
    Permission(String),

    #[error("process not found: pid {0}")]
    NotFound(i32),

    #[error("backend syscall failed: {0}")]
    Backend(#[from] nix::Error),

    #[error("io error: {0}")]
    Io(#[from] io::Error),

    #[error("unknown: {0}")]
    Other(String),
}