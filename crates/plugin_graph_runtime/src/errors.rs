use thiserror::Error;

#[derive(Debug, Error)]
pub enum PluginGraphError {
    #[error("plugin referenced but not found: {0}")]
    UnknownPlugin(String),

    #[error("duplicate plugin id: {0}")]
    DuplicatePlugin(String),

    #[error("cycle detected in plugin graph")]
    CycleDetected,

    #[error("invalid plugin graph structure")]
    InvalidStructure,

    #[error("execution failed: {0}")]
    ExecFailure(String),
}