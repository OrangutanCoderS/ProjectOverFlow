use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Logical identifiers used in replay timelines.
pub type TimelineId = String;
pub type ScenarioId = String;
pub type NodeId = String;
pub type PluginId = String;
pub type DecisionId = String;

/// What kind of event we are replaying.
///
/// Keep this generic so different OverFlow phases can
/// record their own semantics into the payload.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ReplayEventKind {
    Input,
    PluginFired,
    Decision,
    Action,
    Telemetry,
    Error,
}

/// A single recorded event in time.
///
/// `payload` is arbitrary JSON; for divergence analysis
/// we look for payload["baseline_action"] if present.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayEvent {
    pub timeline_id: TimelineId,
    pub sequence_no: u64,
    pub timestamp: DateTime<Utc>,
    pub origin: String,
    pub kind: ReplayEventKind,

    /// Arbitrary structured data recorded at capture time.
    #[serde(default)]
    pub payload: Value,
}

/// Configuration for a replay scenario.
///
/// This is intentionally light; higher layers (Phase V)
/// can layer more metadata on top.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioConfig {
    /// Human readable label.
    pub label: String,
    /// Optional longer description.
    #[serde(default)]
    pub description: String,
    /// If Some(n), cap the number of events processed
    /// by this scenario to n.
    #[serde(default)]
    pub max_events: Option<usize>,
}

/// The outcome of feeding a single event into a decision engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionOutcome {
    pub decision_id: DecisionId,
    pub node_id: Option<NodeId>,
    pub plugin_id: Option<PluginId>,
    /// Canonical action label ("allow", "block", "throttle", etc.).
    pub action: String,
    /// Engine-specific metadata (scores, reasons, extra flags).
    #[serde(default)]
    pub metadata: Value,
}
