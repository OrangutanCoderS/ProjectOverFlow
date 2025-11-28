use thiserror::Error;

/// Errors produced by the trace timeline module.
#[derive(Debug, Error)]
pub enum TraceTimelineError {
    #[error("invalid timestamp: {0}")]
    InvalidTimestamp(String),

    #[error("timeline capacity exceeded: {0} events")]
    CapacityExceeded(usize),

    #[error("inconsistent timeline: {0}")]
    Inconsistent(String),

    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}