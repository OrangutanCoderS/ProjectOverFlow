use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

/// 256-bit hash used for chaining entries.
pub type Hash = [u8; 32];

/// High-level type of the audit event.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", content = "data")]
pub enum AuditEventKind {
    PolicyCreated,
    PolicyUpdated,
    PolicyDeleted,
    PolicyEvaluated,
    PluginGraphChanged,
    EmergencyOverride,
    Custom(String),
}

/// Single semantic event in the audit log.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// Wall-clock timestamp at which the event was recorded.
    pub timestamp: DateTime<Utc>,
    /// Which subsystem / user / node caused this event.
    pub actor: String,
    /// Logical identifier of the policy or entity affected.
    pub policy_id: String,
    /// Event kind.
    pub kind: AuditEventKind,
    /// Arbitrary JSON details (diffs, reasons, before/after, etc.).
    pub details: Value,
}

/// A single entry in the append-only, hash-chained audit log.
///
/// The hash is computed over:
/// - index (big-endian bytes)
/// - prev_hash
/// - serialized `event`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub index: u64,
    pub prev_hash: Hash,
    pub event: AuditEvent,
    pub hash: Hash,
}

impl AuditEntry {
    /// Compute the hash for a given (index, prev_hash, event) triple.
    pub fn compute_hash(index: u64, prev_hash: &Hash, event: &AuditEvent) -> Hash {
        let mut hasher = Sha256::new();

        hasher.update(index.to_be_bytes());
        hasher.update(prev_hash);

        let serialized =
            serde_json::to_vec(event).expect("serializing AuditEvent for hash computation");
        hasher.update(&serialized);

        let digest = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&digest);
        hash
    }
}
