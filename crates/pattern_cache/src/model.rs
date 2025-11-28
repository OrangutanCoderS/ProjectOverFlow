use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A mined pattern from telemetry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pattern {
    pub id: i64,
    pub category: String,
    pub fingerprint: String,
    pub score: f64,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
}

/// Used when inserting patterns (ID is assigned by DB)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewPattern {
    pub category: String,
    pub fingerprint: String,
    pub score: f64,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
}
