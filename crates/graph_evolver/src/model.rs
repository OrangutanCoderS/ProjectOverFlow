use serde::{Deserialize, Serialize};

/// Simple string-based node ID for portability across crates.
pub type NodeId = String;

/// Minimal node view for evolution decisions.
/// Real node metadata can live elsewhere; this is what
/// the evolver actually needs.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PluginNode {
    pub id: NodeId,
    pub label: String,
    /// Trust / quality score in [0, 1] (caller-defined semantics).
    pub trust: f64,
}

/// Directed edge in the plugin graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub from: NodeId,
    pub to: NodeId,
    /// Weight used by runtime routing (caller-defined semantics).
    pub weight: f64,
}

/// The graph view the evolver operates on.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginGraph {
    pub nodes: Vec<PluginNode>,
    pub edges: Vec<Edge>,
}

impl PluginGraph {
    /// Return true if an edge (from, to) already exists.
    pub fn has_edge(&self, from: &NodeId, to: &NodeId) -> bool {
        self.edges
            .iter()
            .any(|e| &e.from == from && &e.to == to)
    }

    /// Append an edge without any validation.
    pub fn add_edge(&mut self, from: NodeId, to: NodeId, weight: f64) {
        self.edges.push(Edge { from, to, weight });
    }
}

/// Aggregated telemetry stats used both for scoring and eligibility.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetrySummary {
    pub support_count: u64,
    pub avg_latency_ms: f64,
    pub error_rate: f64,
}

/// A single suggested edge from pattern mining.
/// The evolver decides whether to accept it and with what weight.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestedEdge {
    pub from: NodeId,
    pub to: NodeId,
    pub stats: TelemetrySummary,
}

/// Scalar score for a candidate graph plus the stats driving it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphScore {
    pub score: f64,
    pub support_count: u64,
    pub avg_latency_ms: f64,
    pub error_rate: f64,
}

/// What kind of change we applied to obtain a candidate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvolutionOpKind {
    AddEdge,
    RemoveEdge,
    ReweightEdge,
}

/// A single evolution operation applied to the base graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionOp {
    pub kind: EvolutionOpKind,
    pub from: Option<NodeId>,
    pub to: Option<NodeId>,
    /// For AddEdge and ReweightEdge, the new or delta weight.
    pub delta_weight: Option<f64>,
    /// Human-readable reason for audit / debugging.
    pub reason: String,
}

/// A concrete candidate graph produced by the evolver.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionCandidate {
    pub id: String,
    pub graph: PluginGraph,
    pub score: GraphScore,
    pub operations: Vec<EvolutionOp>,
}
