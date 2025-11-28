use thiserror::Error;

/// Errors emitted by the policy audit log.
#[derive(Debug, Error)]
pub enum PolicyAuditError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    /// Any structural or cryptographic inconsistency in the log.
    #[error("inconsistent audit log: {0}")]
    Inconsistent(String),
}
