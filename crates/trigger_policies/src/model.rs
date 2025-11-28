use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// One trigger policy rule, directly mirroring YAML schema v1.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerPolicy {
    pub id: String,
    pub trigger_name: String,
    pub scope: String,        // "process", "system", etc.
    pub conditions: HashMap<String, String>, // key-value or operator spec
    pub actions: Vec<String>, // list of plugin actions
    pub priority: u32,
    pub cooldown: u64,        // seconds
    #[serde(default)]
    pub suppress_if: Option<String>,
    #[serde(default)]
    pub require_if: Option<String>,
    #[serde(default = "default_mode")]
    pub mode: String,         // "enforce"|"simulate"
    #[serde(default)]
    pub notes: Option<String>,
}

fn default_mode() -> String { "enforce".to_string() }

/// Root structure when loading multiple policies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicySet {
    pub policies: Vec<TriggerPolicy>,
}