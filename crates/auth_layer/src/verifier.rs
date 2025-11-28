use std::{
    num::NonZeroUsize,
    time::{Duration, SystemTime},
};

use base64::{engine::general_purpose::STANDARD, Engine as _};
use chrono::{DateTime, Utc};
use ed25519_dalek::{Signature, Verifier};
use lru::LruCache;
use parking_lot::Mutex;
use tracing::warn;

use crate::{
    error::{AuthError, AuthResult},
    keystore::PeerKeyStore,
    model::AuthToken,
};

/// Stateless-ish authentication facade:
/// - validates timestamp skew
/// - rejects replays via (peer_id, nonce)
/// - verifies Ed25519 signature against known public keys
pub struct AuthLayer {
    keys: PeerKeyStore,
    nonces: Mutex<LruCache<(String, String), ()>>, // (peer_id, nonce)
    max_skew: Duration,
}

impl AuthLayer {
    /// `keys`      : trusted public keys
    /// `capacity`  : how many (peer_id, nonce) entries to remember for replay protection
    /// `max_skew`  : maximum allowed clock skew between token timestamp and now
    pub fn new(keys: PeerKeyStore, capacity: usize, max_skew: Duration) -> Self {
        let cap = NonZeroUsize::new(capacity.max(1)).unwrap();
        Self {
            keys,
            nonces: Mutex::new(LruCache::new(cap)),
            max_skew,
        }
    }

    /// Validate an `AuthToken` against the given raw payload bytes.
    ///
    /// Steps:
    /// 1. Parse timestamp and enforce skew.
    /// 2. Check replay via (peer_id, nonce) in LRU.
    /// 3. Lookup peer public key.
    /// 4. Decode base64 signature and verify Ed25519.
    pub fn validate_token(&self, token: &AuthToken, payload: &[u8]) -> AuthResult<()> {
        self.validate_timestamp(token)?;
        self.check_replay(token)?;
        self.verify_signature(token, payload)?;
        Ok(())
    }

    fn validate_timestamp(&self, token: &AuthToken) -> AuthResult<()> {
        let ts: DateTime<Utc> = token.timestamp.parse()?;

        // Convert chrono DateTime<Utc> to SystemTime without relying on unstable helpers.
        let secs = ts.timestamp();
        let nanos = ts.timestamp_subsec_nanos();

        if secs < 0 {
            // Extremely old / invalid; we treat as skew.
            return Err(AuthError::ClockSkew);
        }

        let packet_time =
            SystemTime::UNIX_EPOCH + Duration::from_secs(secs as u64) + Duration::from_nanos(nanos as u64);

        let now = SystemTime::now();
        let diff = now
            .duration_since(packet_time)
            .map_err(|_| AuthError::ClockSkew)?;

        if diff > self.max_skew {
            warn!(
                "auth_layer: clock skew exceeded: {:?} > {:?}",
                diff, self.max_skew
            );
            return Err(AuthError::ClockSkew);
        }

        Ok(())
    }

    fn check_replay(&self, token: &AuthToken) -> AuthResult<()> {
        let key = (token.peer_id.clone(), token.nonce.clone());

        let mut cache = self.nonces.lock();
        if cache.contains(&key) {
            warn!(
                "auth_layer: replay detected for peer_id={} nonce={}",
                token.peer_id, token.nonce
            );
            return Err(AuthError::Replay);
        }

        cache.put(key, ());
        Ok(())
    }

    fn verify_signature(&self, token: &AuthToken, payload: &[u8]) -> AuthResult<()> {
        let vk = self
            .keys
            .get(&token.peer_id)
            .ok_or_else(|| AuthError::UnknownPeer(token.peer_id.clone()))?;

        let sig_bytes = STANDARD.decode(&token.signature_b64)?;

        if sig_bytes.len() != ed25519_dalek::SIGNATURE_LENGTH {
            return Err(AuthError::BadSignatureLen(sig_bytes.len()));
        }

        let arr: [u8; ed25519_dalek::SIGNATURE_LENGTH] = sig_bytes
            .as_slice()
            .try_into()
            .expect("length already checked");

        let sig = Signature::from_bytes(&arr);

        vk.verify(payload, &sig)?;

        Ok(())
    }
}
