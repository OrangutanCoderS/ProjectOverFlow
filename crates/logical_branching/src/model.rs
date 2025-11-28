use hashbrown::{HashMap, HashSet};
use plugin_response_chain::ActionId;
use serde::{Deserialize, Serialize};

/// Expected plugin state matcher as declared in YAML.
///
/// Example:
/// ```yaml
/// plugin_states:
///   fake_net: active
///   entropy_flagger: idle
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PluginStateMatch {
    Active,
    Idle,
    Disabled,
    /// For future custom states
    Other(String),
}

/// One logical rule from logical_branching.yaml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogicalRule {
    /// Required context flags (all must be present).
    #[serde(default)]
    pub context_conditions: Vec<String>,

    /// Required plugin states (all must match).
    /// plugin_name -> expected_state
    #[serde(default)]
    pub plugin_states: HashMap<String, PluginStateMatch>,

    /// Required system modifiers (all must match).
    /// e.g., {"secure_mode": "off"}
    #[serde(default)]
    pub system_modifiers: HashMap<String, String>,

    /// Whitelist: if non-empty, candidate actions are intersected with this set.
    #[serde(default)]
    pub allowed_responses: Vec<ActionId>,

    /// Blacklist: actions that are always removed from the candidate set.
    #[serde(default)]
    pub blocked_paths: Vec<ActionId>,

    /// Fallback chain if nothing survives allowed/blocked filtering.
    #[serde(default)]
    pub fallback: Vec<ActionId>,
}

/// Top-level config wrapper for logical_branching.yaml.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogicalBranchingConfig {
    #[serde(default)]
    pub rules: Vec<LogicalRule>,
}

/// Runtime view of actual plugin state (not the YAML matcher).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PluginRuntimeState {
    Active,
    Idle,
    Disabled,
    Other(String),
}

/// Snapshot of world-state used for matching a rule.
#[derive(Debug, Clone)]
pub struct RuntimeSnapshot {
    /// Context flags like ["high_memory_pressure", "usb_spike"]
    pub context_flags: HashSet<String>,

    /// plugin_name -> runtime state
    pub plugin_states: HashMap<String, PluginRuntimeState>,

    /// System modifiers like {"secure_mode": "off"}
    pub system_modifiers: HashMap<String, String>,
}

/// Decision produced by the logical branching engine.
#[derive(Debug, Clone)]
pub struct BranchDecision {
    /// Final actions the system is allowed to execute.
    pub allowed: Vec<ActionId>,
    /// Actions that were explicitly removed by a rule.
    pub blocked: Vec<ActionId>,
    /// If a fallback chain was used, this contains it.
    pub fallback_used: Option<Vec<ActionId>>,
    /// Index of the rule that determined this decision (if any).
    pub matched_rule_index: Option<usize>,
}