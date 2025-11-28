use thiserror::Error;

/// Unified error type for trigger_thresholds
#[derive(Debug, Error)]
pub enum ThresholdError {
    #[error("Config parse error: {0}")]
    ConfigParse(String),

    #[error("Invalid threshold value for key: {0}")]
    InvalidValue(String),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Runtime evaluation error: {0}")]
    Evaluation(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}