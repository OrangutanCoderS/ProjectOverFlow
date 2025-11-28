use ed25519_dalek::{Signer, SigningKey};
use rand::rngs::OsRng;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;

pub struct PeerSigner {
    key: SigningKey,
}

impl PeerSigner {
    pub fn generate() -> Self {
        let mut csprng = OsRng;
        let key = SigningKey::generate(&mut csprng);
        Self { key }
    }

    pub fn sign(&self, data: &[u8]) -> String {
        let sig = self.key.sign(data);
        STANDARD.encode(sig.to_bytes())
    }

    pub fn public_key_bytes(&self) -> Vec<u8> {
        self.key.verifying_key().to_bytes().to_vec()
    }
}