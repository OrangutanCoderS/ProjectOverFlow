use thiserror::Error;

/// Top-level error type for the plugin orchestrator.
///
/// This intentionally stays generic for now. As you start
/// wiring in the other crates, you can add variants that wrap
/// their error types instead of just using `String`.
#[derive(Debug, Error)]
pub enum OrchestratorError {
    #[error("configuration error: {0}")]
    Config(String),

    #[error("initialization error: {0}")]
    Init(String),

    #[error("runtime error: {0}")]
    Runtime(String),
}