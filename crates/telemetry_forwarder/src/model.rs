use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// High-level category of telemetry. This is deliberately small and stable.
///
/// Downstream systems can route differently based on category.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TelemetryCategory {
    Cpu,
    Memory,
    Disk,
    Network,
    ProcessEvent,
    PluginAlert,
    PolicyViolation,
    Heartbeat,
    Custom,
}

/// Raw telemetry payload coming from system/daemon code.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryPayload {
    /// Logical category for routing / dashboards.
    pub category: TelemetryCategory,
    /// Arbitrary structured payload.
    pub body: Value,
}

/// Canonical envelope that goes out over the mesh / bridge.
///
/// This is what gets serialized, optionally signed, and shipped.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryEnvelope {
    /// Node that originated this telemetry.
    pub origin_id: String,
    /// RFC3339 timestamp of when the envelope was created.
    pub timestamp: DateTime<Utc>,
    /// Optional correlation id for traces / plugin chains.
    pub trace_id: Option<String>,
    /// Logical telemetry category.
    pub category: TelemetryCategory,
    /// Raw payload body.
    pub body: Value,
    /// Optional auth field where upper layers can store signatures / tokens.
    pub auth: Option<Value>,
}

/// Result meta about a forwarded telemetry event.
///
/// For now this is minimal; later this can carry ack ids, latency, etc.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForwardResult {
    /// Whether the send was queued/sent successfully.
    pub accepted: bool,
    /// Optional human-readable detail (for logs / debugging).
    pub detail: Option<String>,
}

impl ForwardResult {
    pub fn accepted(detail: impl Into<Option<String>>) -> Self {
        Self {
            accepted: true,
            detail: detail.into(),
        }
    }

    pub fn rejected(detail: impl Into<Option<String>>) -> Self {
        Self {
            accepted: false,
            detail: detail.into(),
        }
    }
}
