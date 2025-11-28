use thiserror::Error;

/// Errors that can occur while loading or compiling plugin_response_chain.json
#[derive(Debug, Error)]
pub enum PluginChainError {
    #[error("I/O error while reading config: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON parse error in plugin_response_chain: {0}")]
    Json(#[from] serde_json::Error),

    // YAML reserved for future; keep variant now so signature is stable.
    #[error("YAML parse error in plugin_response_chain: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("plugin `{0}` has no severity rules defined")]
    EmptyPlugin(String),

    #[error("plugin `{plugin}` severity `{severity}` has no pattern rules defined")]
    EmptySeverity {
        plugin: String,
        severity: String,
    },

    #[error(
        "unknown action `{action}` for plugin `{plugin}` severity `{severity}` pattern `{pattern}`"
    )]
    UnknownAction {
        plugin: String,
        severity: String,
        pattern: String,
        action: String,
    },
}