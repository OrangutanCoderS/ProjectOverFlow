use std::collections::HashMap;

use ed25519_dalek::{VerifyingKey, PUBLIC_KEY_LENGTH};
use parking_lot::RwLock;

use crate::error::{AuthError, AuthResult};

/// Thread-safe store of trusted peer public keys.
pub struct PeerKeyStore {
    inner: RwLock<HashMap<String, VerifyingKey>>,
}

impl PeerKeyStore {
    /// Create a store from an existing map.
    pub fn from_map(map: HashMap<String, VerifyingKey>) -> Self {
        Self {
            inner: RwLock::new(map),
        }
    }

    /// Create a store from raw public key bytes.
    ///
    /// `raw` maps peer_id -> 32-byte Ed25519 public key.
    pub fn from_bytes_map(raw: &HashMap<String, Vec<u8>>) -> AuthResult<Self> {
        let mut map = HashMap::new();

        for (peer_id, bytes) in raw {
            if bytes.len() != PUBLIC_KEY_LENGTH {
                return Err(AuthError::BadPublicKeyLen(bytes.len()));
            }

            let arr: [u8; PUBLIC_KEY_LENGTH] = bytes
                .as_slice()
                .try_into()
                .expect("length already checked above");

            let vk = VerifyingKey::from_bytes(&arr)?;
            map.insert(peer_id.clone(), vk);
        }

        Ok(Self::from_map(map))
    }

    /// Lookup a peer's public key. Returns a clone so caller can't mutate internal state.
    pub fn get(&self, peer_id: &str) -> Option<VerifyingKey> {
        self.inner.read().get(peer_id).cloned()
    }

    /// Insert or update a peer's key at runtime (e.g., key rotation).
    pub fn insert(&self, peer_id: String, key: VerifyingKey) {
        self.inner.write().insert(peer_id, key);
    }

    /// Number of keys currently stored (mostly for tests / metrics).
    pub fn len(&self) -> usize {
        self.inner.read().len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
