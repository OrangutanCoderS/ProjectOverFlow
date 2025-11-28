use rand::RngCore;

/// 32-byte random salt used for hashing sensitive values.
/// This must be generated once per daemon run and kept only in memory.
#[derive(Debug, Clone)]
pub struct Salt {
    bytes: [u8; 32],
}

impl Salt {
    /// Create a new Salt from raw bytes.
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Salt { bytes }
    }

    /// Generate a new cryptographically-random salt.
    pub fn random() -> Self {
        let mut b = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut b);
        Salt { bytes: b }
    }

    /// Borrow salt as byte slice.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.bytes
    }
}
