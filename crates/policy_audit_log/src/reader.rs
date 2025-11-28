use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use serde_json;
use sha2::{Digest, Sha256};

use crate::error::PolicyAuditError;
use crate::model::{AuditEntry, AuditEvent, Hash};

/// Read and verify audit logs written by AuditLogWriter.
pub struct AuditLogReader;

impl AuditLogReader {
    /// Load all entries from `path` and verify the hash chain & indices.
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Vec<AuditEntry>, PolicyAuditError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let mut entries = Vec::new();
        for line in reader.lines() {
            let line = line?;
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let entry: AuditEntry = serde_json::from_str(line)?;
            entries.push(entry);
        }

        Self::verify_chain(&entries)?;
        Ok(entries)
    }

    /// Verify index monotonicity and hash chain consistency.
    pub fn verify_chain(entries: &[AuditEntry]) -> Result<(), PolicyAuditError> {
        let mut expected_prev: Hash = [0u8; 32];

        for (i, entry) in entries.iter().enumerate() {
            let expected_index = i as u64;

            // 1. Index mismatch
            if entry.index != expected_index {
                return Err(PolicyAuditError::Inconsistent(format!(
                    "index mismatch at entry {} (expected {}, got {})",
                    entry.index, expected_index, entry.index
                )));
            }

            // 2. Previous hash mismatch
            if entry.prev_hash != expected_prev {
                return Err(PolicyAuditError::Inconsistent(format!(
                    "hash chain broken at entry {}: prev_hash mismatch",
                    entry.index
                )));
            }

            // 3. Recompute current hash
            let recomputed = Self::compute_hash(entry.index, &entry.prev_hash, &entry.event)?;
            if recomputed != entry.hash {
                return Err(PolicyAuditError::Inconsistent(format!(
                    "hash mismatch at entry {}",
                    entry.index
                )));
            }

            expected_prev = entry.hash;
        }

        Ok(())
    }

    fn compute_hash(
        index: u64,
        prev_hash: &Hash,
        event: &AuditEvent,
    ) -> Result<Hash, PolicyAuditError> {
        let mut hasher = Sha256::new();
        hasher.update(index.to_le_bytes());
        hasher.update(prev_hash);
        hasher.update(serde_json::to_vec(event)?);
        Ok(hasher.finalize().into())
    }
}