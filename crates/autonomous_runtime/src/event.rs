use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Generic event flowing *into* the runtime from observation modules.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleEvent {
    /// Which subsystem produced this event (e.g. "system/cpu", "netmon").
    pub source: String,

    /// Logical event type, stable for routing (e.g. "high_cpu", "file_created").
    pub kind: String,

    /// UTC timestamp of when the event was produced.
    pub timestamp: DateTime<Utc>,

    /// Arbitrary JSON payload, schema defined by the producer.
    pub payload: serde_json::Value,
}

/// Action emitted *out* of the runtime towards actuators / modules.
///
/// This is intentionally generic – concrete crates adapt their own
/// request types to/from this structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeAction {
    /// Logical target (e.g. "action_throttle", "action_kill", "securemode").
    pub target: String,

    /// Logical action name (e.g. "throttle", "kill", "enter_secure_mode").
    pub kind: String,

    /// Arbitrary JSON parameters.
    pub parameters: serde_json::Value,
}