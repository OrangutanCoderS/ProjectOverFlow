use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A normalized telemetry event used for pattern mining.
///
/// You will map raw logs (from plugin_engine, graph_runtime, etc.)
/// into this shape before handing them to the miner.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryEvent {
    /// When this event occurred.
    pub timestamp: DateTime<Utc>,

    /// Optional session identifier (e.g., trace id, replay id).
    pub session_id: Option<String>,

    /// Process id, if known.
    pub process_id: Option<i64>,

    /// Origin plugin that produced the event, if relevant.
    pub origin_plugin: Option<String>,

    /// The plugin whose activation we are modeling in the chain.
    pub plugin: String,

    /// Result status ("ok", "suspicious", "blocked", ...).
    pub result: Option<String>,

    /// Optional entropy score for the event payload (if available).
    pub entropy: Option<f64>,

    /// Whether this event corresponds to a failure / error outcome.
    pub failed: bool,
}

/// Aggregate stats for one mined pattern/motif.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternStats {
    /// SHA-256 fingerprint of the plugin chain (plugin names sequence).
    pub fingerprint: String,

    /// Number of plugins in this chain.
    pub length: u32,

    /// First time we saw this pattern.
    pub first_seen: DateTime<Utc>,

    /// Most recent time we saw this pattern.
    pub last_seen: DateTime<Utc>,

    /// Number of times this pattern appeared (support).
    pub count: u64,

    /// Average entropy across events in this pattern (if tracked).
    pub avg_entropy: Option<f64>,

    /// Approximate failure likelihood (failures / count).
    pub failure_rate: Option<f64>,

    /// Example plugin chain (ordered list of plugin names).
    pub sample_chain: Vec<String>,
}