use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Node identifier in the plugin DAG.
/// Keep as String to align with JSON graph representations.
pub type NodeId = String;

/// Single plugin node in the execution DAG.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PluginNode {
    /// Unique node id inside the DAG.
    pub id: NodeId,
    /// Logical plugin identity (e.g. "cpu-usage-filter").
    pub plugin_id: String,
    /// Monotonic version (used for conflict resolution).
    pub version: u32,
    /// Remote trust score [0.0, 1.0].
    pub trust: f32,
    /// Optional tags (e.g. ["remote", "experimental"]).
    #[serde(default)]
    pub tags: Vec<String>,
    /// Arbitrary metadata blob (config, owner, etc.).
    #[serde(default)]
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Edge {
    pub from: NodeId,
    pub to: NodeId,
    #[serde(default)]
    pub label: Option<String>,
}

/// Simple DAG representation.
/// Edges are directed: from -> to.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginGraph {
    pub nodes: HashMap<NodeId, PluginNode>,
    pub edges: Vec<Edge>,
}

impl PluginGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
        }
    }

    pub fn with_capacity(node_cap: usize, edge_cap: usize) -> Self {
        Self {
            nodes: HashMap::with_capacity(node_cap),
            edges: Vec::with_capacity(edge_cap),
        }
    }

    /// Insert or replace a node. Returns previous node if existed.
    pub fn insert_node(&mut self, node: PluginNode) -> Option<PluginNode> {
        self.nodes.insert(node.id.clone(), node)
    }

    pub fn add_edge(&mut self, edge: Edge) {
        self.edges.push(edge);
    }

    pub fn contains_node(&self, id: &str) -> bool {
        self.nodes.contains_key(id)
    }
}

/// Merge policy controls what remote changes are allowed.
#[derive(Debug, Clone)]
pub struct MergePolicy {
    /// Minimum trust score required for a remote node to be considered.
    pub min_trust: f32,

    /// Allow replacing local nodes with higher or equal version.
    pub allow_overwrite: bool,

    /// Allow replacing local node with *lower* version.
    /// If false, lower versions are treated as conflicts.
    pub allow_downgrade: bool,

    /// Optional limit on number of brand new nodes (None = unlimited).
    pub max_new_nodes: Option<usize>,

    /// Optional limit on number of new edges (None = unlimited).
    pub max_new_edges: Option<usize>,
}

impl Default for MergePolicy {
    fn default() -> Self {
        Self {
            min_trust: 0.5,
            allow_overwrite: true,
            allow_downgrade: false,
            max_new_nodes: None,
            max_new_edges: None,
        }
    }
}

/// Summary of a merge operation for telemetry/debug.
#[derive(Debug, Default, Clone)]
pub struct MergeReport {
    pub nodes_added: usize,
    pub nodes_updated: usize,
    pub nodes_skipped_low_trust: usize,
    pub nodes_skipped_limit: usize,
    pub conflicts_version: usize,

    pub edges_added: usize,
    pub edges_skipped_cycle: usize,
    pub edges_skipped_missing_node: usize,
    pub edges_skipped_limit: usize,
}