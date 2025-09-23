use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Minimal, future-proof packet.
/// `metrics` is a free-form JSON object so producers don't link on our schema.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryPacket {
    pub timestamp: DateTime<Utc>,
    pub source: String,            // e.g. "system_stats", "process_monitor"
    pub metrics: Value,            // arbitrary structured payload
}

impl TelemetryPacket {
    /// Helper constructor with current timestamp.
    pub fn new<S: Into<String>>(source: S, metrics: Value) -> Self {
        Self {
            timestamp: Utc::now(),
            source: source.into(),
            metrics,
        }
    }
}
