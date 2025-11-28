use serde::{Deserialize, Serialize};
use bytes::Bytes;

/// Fundamental sync message between nodes.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SyncMessage {
    pub origin_id: String,
    pub timestamp: String,
    pub payload: serde_json::Value,
    pub signature: String, // base64
}

impl SyncMessage {
    pub fn raw_bytes(&self) -> Bytes {
        Bytes::from(
            serde_json::to_vec(&self)
                .expect("SyncMessage serialization must not fail"),
        )
    }
}
