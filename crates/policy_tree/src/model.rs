use serde::{Deserialize, Serialize};

/// Maximum supported nodes in a single policy tree.
pub const MAX_POLICY_NODES: usize = 8192;

/// How a single condition is expressed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConditionKind {
    /// Numeric threshold on a metric: min <= value <= max (if provided).
    Threshold {
        metric: String,
        min: Option<f32>,
        max: Option<f32>,
    },
    /// Requires a single tag to be present.
    TagAny { tag: String },
    /// Requires all tags in the list to be present.
    TagAll { tags: Vec<String> },
    /// Active only if current timestamp in [start_ms, end_ms].
    TimeRange {
        start_ms: u64,
        end_ms: u64,
    },
}

/// A set of conditions, split into AND and OR groups.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConditionSet {
    /// All of these MUST match.
    #[serde(default)]
    pub all: Vec<ConditionKind>,
    /// At least one (if non-empty) MUST match.
    #[serde(default)]
    pub any: Vec<ConditionKind>,
}

/// Leaf action at a policy node.
/// This is intentionally minimal; extend as Phase 3–4 need.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyActionDef {
    pub name: String,
    pub severity: u8,
    #[serde(default)]
    pub labels: Vec<String>,
}

/// One node definition in the tree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyNodeDef {
    /// Unique node id within this tree.
    pub id: String,

    /// Optional conditions for this node. If None, node is always eligible.
    pub conditions: Option<ConditionSet>,

    /// Child node ids – these must exist in `PolicyTreeDef.nodes`.
    #[serde(default)]
    pub children: Vec<String>,

    /// Optional action if node is a leaf (or mid-node with side-effect).
    #[serde(default)]
    pub action: Option<PolicyActionDef>,
}

/// Full policy tree definition loaded from configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyTreeDef {
    /// Id of the root node.
    pub root_id: String,

    /// All nodes in this tree (order arbitrary).
    pub nodes: Vec<PolicyNodeDef>,
}