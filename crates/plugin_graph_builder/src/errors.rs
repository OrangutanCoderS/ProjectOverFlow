use thiserror::Error;

#[derive(Debug, Error)]
pub enum PluginGraphBuilderError {
    #[error("Invalid plugin reference: `{0}`")]
    InvalidPluginRef(String),

    #[error("Cycle detected in plugin graph")]
    CycleDetected,

    #[error("Duplicate plugin ID: `{0}`")]
    DuplicatePlugin(String),

    #[error("Missing plugin: `{0}`")]
    MissingPlugin(String),

    #[error("JSON/YAML parse error: {0}")]
    ParseError(String),
}
