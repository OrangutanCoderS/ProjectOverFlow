use thiserror::Error;

#[derive(Debug, Error)]
pub enum TelemetryError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("Invalid input: {0}")]
    Invalid(&'static str),
}
