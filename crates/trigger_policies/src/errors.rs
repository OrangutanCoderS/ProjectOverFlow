use thiserror::Error;

/// Comprehensive policy-engine errors (typed, no Strings).
#[derive(Debug, Error)]
pub enum PolicyError {
    #[error("YAML parse failed: {0}")]
    Parse(#[from] serde_yaml::Error),

    #[error("Condition evaluation failed: {0}")]
    Eval(String),

    #[error("Invalid field in policy: {0}")]
    InvalidField(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}