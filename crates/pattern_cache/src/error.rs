use thiserror::Error;

/// Pattern Cache errors
#[derive(Debug, Error)]
pub enum PatternCacheError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("DB error: {0}")]
    Db(#[from] rusqlite::Error),

    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("inconsistent or corrupt state: {0}")]
    Inconsistent(String),
}
