use thiserror::Error;

/// Errors emitted by telemetry_pattern_miner.
#[derive(Debug, Error)]
pub enum TelemetryPatternError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("database error: {0}")]
    Db(#[from] rusqlite::Error),

    #[error("time parse error: {0}")]
    Time(String),

    #[error("inconsistent data: {0}")]
    Inconsistent(String),

    #[error("other: {0}")]
    Other(String),
}