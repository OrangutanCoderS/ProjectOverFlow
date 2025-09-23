use thiserror::Error;

#[derive(Debug, Error)]
pub enum WatchdogError {
    #[error("unit already exists: {0}")]
    DuplicateUnit(String),

    #[error("unit not found: {0}")]
    UnknownUnit(String),

    #[error("invalid config: {0}")]
    InvalidConfig(&'static str),

    #[error("restart callback failed: {0}")]
    RestartFailure(String),

    #[cfg(feature = "logging")]
    #[error("logs error: {0}")]
    Logs(#[from] logs::error::LogError),

    #[cfg(feature = "telemetry")]
    #[error("telemetry error: {0}")]
    Telemetry(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}
