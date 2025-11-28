use serde::{Deserialize, Serialize};

/// Canonical auth token that rides alongside mesh packets.
///
/// This does NOT include the payload; the payload is signed separately.
/// This struct is what we validate against the raw payload bytes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthToken {
    /// Logical peer id (e.g., "node-1", "daemon-A")
    pub peer_id: String,
    /// RFC3339 timestamp when the token was created.
    pub timestamp: String,
    /// Caller-chosen nonce; must be unique per (peer_id, timestamp).
    pub nonce: String,
    /// Base64-encoded Ed25519 signature over the raw payload bytes.
    pub signature_b64: String,
}
