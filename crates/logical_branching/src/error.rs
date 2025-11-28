use thiserror::Error;

/// Errors for loading/validating logical_branching.yaml and building the engine.
#[derive(Debug, Error)]
pub enum LogicalBranchError {
    #[error("YAML parse error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Contradictory action {action} in rule {rule_index}")]
    ContradictoryAction { action: String, rule_index: usize },

    #[error("Unknown action {action} in rule {rule_index}")]
    UnknownAction { action: String, rule_index: usize },

    #[error("Logical branching config is empty")]
    EmptyConfig,
}