use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

use serde_json;
use sha2::{Digest, Sha256};

use crate::error::PolicyAuditError;
use crate::model::{AuditEntry, AuditEvent, Hash};

/// Append-only audit log writer with hash chaining.
///
/// Format: one JSON-serialized `AuditEntry` per line.
/// Hash chain:
///   H_i = SHA256( index_i || prev_hash_i || serde_json(event_i) )
pub struct AuditLogWriter {
    path: PathBuf,
    writer: BufWriter<File>,
    last_hash: Hash,
    next_index: u64,
}

impl AuditLogWriter {
    /// Create a *fresh* audit log at `path`.
    /// If the file already exists, it is truncated.
    ///
    /// macOS-safe: we only open with `write` and `truncate`, no `read+append`.
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, PolicyAuditError> {
        let path_ref = path.as_ref();

        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path_ref)?;

        Ok(Self {
            path: path_ref.to_path_buf(),
            writer: BufWriter::new(file),
            last_hash: [0u8; 32],
            next_index: 0,
        })
    }

    /// Append a single event, update hash chain, and flush to disk.
    pub fn append(&mut self, event: AuditEvent) -> Result<AuditEntry, PolicyAuditError> {
        let index = self.next_index;
        let prev_hash = self.last_hash;

        // Compute hash H_i = SHA256(index || prev_hash || serialized(event))
        let mut hasher = Sha256::new();
        hasher.update(index.to_le_bytes());
        hasher.update(&prev_hash);
        let event_bytes = serde_json::to_vec(&event)?;
        hasher.update(&event_bytes);
        let hash: [u8; 32] = hasher.finalize().into();

        let entry = AuditEntry {
            index,
            event,
            prev_hash,
            hash,
        };

        // JSON line
        let line = serde_json::to_vec(&entry)?;
        self.writer.write_all(&line)?;
        self.writer.write_all(b"\n")?;
        self.writer.flush()?; // ensure tests/benches see bytes immediately

        // Advance chain state
        self.last_hash = entry.hash;
        self.next_index += 1;

        Ok(entry)
    }

    /// Path backing this writer (mainly for debugging/tests).
    pub fn path(&self) -> &Path {
        &self.path
    }
}