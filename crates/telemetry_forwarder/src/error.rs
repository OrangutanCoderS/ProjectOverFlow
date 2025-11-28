use thiserror::Error;

/// Errors that can occur while building or forwarding telemetry.
#[derive(Debug, Error)]
pub enum TelemetryError {
    #[error("queue is full; telemetry dropped")]
    QueueFull,

    #[error("sink failure: {0}")]
    SinkFailure(String),

    #[error("serialization error: {0}")]
    SerializationError(String),

    #[error("invalid payload: {0}")]
    InvalidPayload(&'static str),
}
