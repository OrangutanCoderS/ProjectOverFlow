use thiserror::Error;

/// Errors emitted by the graph evolution engine.
#[derive(Debug, Error)]
pub enum GraphEvolverError {
    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("invalid graph: {0}")]
    InvalidGraph(String),
}
