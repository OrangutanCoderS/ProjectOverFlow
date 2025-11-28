use thiserror::Error;

/// Errors that can occur when constructing or running the
/// memory replay engine.
#[derive(Debug, Error)]
pub enum MemoryReplayError {
    #[error("timeline is empty")]
    EmptyTimeline,

    #[error("invalid timeline ordering at index {index}")]
    InvalidOrder { index: usize },

    #[error("internal error: {0}")]
    Internal(String),
}
