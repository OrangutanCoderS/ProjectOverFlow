use thiserror::Error;

#[derive(Debug, Error)]
pub enum ContextGraphError {
    #[error("node limit exceeded: attempted {attempted}, max {max}")]
    NodeLimitExceeded { attempted: usize, max: usize },

    #[error("duplicate node key encountered")]
    DuplicateNodeKey,

    #[error("unknown node key")]
    UnknownNodeKey,

    #[error("cycle detected in graph")]
    CycleDetected,

    #[error("invalid weight: {0}")]
    InvalidWeight(f32),

    #[error("invalid threshold: {msg}")]
    InvalidThreshold { msg: String },

    #[error("propagation depth exceeded max_depth")]
    DepthExceeded,

    #[error("parse error: {msg}")]
    ParseError { msg: String },
}