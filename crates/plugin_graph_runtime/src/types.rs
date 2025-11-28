use serde::{Serialize, Deserialize};

pub type PluginId = String;
pub type PluginIndex = u16;

/// Input context for plugin execution.
#[derive(Debug, Clone)]
pub struct ExecContext {
    pub event: String,
    pub metadata: serde_json::Value,
}

/// Plugins return diffs only — never full state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextDelta {
    pub new_events: Vec<String>,
    pub warnings: Vec<String>,
    pub metadata_updates: serde_json::Value,
}