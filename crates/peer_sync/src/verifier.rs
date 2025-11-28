use ed25519_dalek::{Verifier, VerifyingKey, Signature};
use crate::error::PeerSyncError;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;

pub struct PeerVerifier {
    pub_key: VerifyingKey,
}

impl PeerVerifier {
    pub fn new(bytes: &[u8]) -> Result<Self, PeerSyncError> {
        if bytes.len() != 32 {
            return Err(PeerSyncError::BadSignature);
        }

        let mut key_bytes = [0u8; 32];
        key_bytes.copy_from_slice(bytes);

        let key = VerifyingKey::from_bytes(&key_bytes)
            .map_err(|_| PeerSyncError::BadSignature)?;

        Ok(Self { pub_key: key })
    }

    pub fn verify(&self, data: &[u8], sig_b64: &str) -> Result<(), PeerSyncError> {
        let decoded = STANDARD
            .decode(sig_b64)
            .map_err(|_| PeerSyncError::BadSignature)?;

        if decoded.len() != 64 {
            return Err(PeerSyncError::BadSignature);
        }

        let mut sig_bytes = [0u8; 64];
        sig_bytes.copy_from_slice(&decoded);

        let sig = Signature::from_bytes(&sig_bytes);

        self.pub_key
            .verify(data, &sig)
            .map_err(|_| PeerSyncError::BadSignature)
    }
}