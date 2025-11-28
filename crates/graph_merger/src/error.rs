use thiserror::Error;

#[derive(Debug, Error)]
pub enum GraphMergeError {
    /// Local or remote graph is structurally invalid (e.g. self-loop).
    #[error("inconsistent graph structure: {0}")]
    Inconsistent(String),

    /// Generic internal error.
    #[error("internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, GraphMergeError>;