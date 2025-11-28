use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A single file entry within a snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotFile {
    /// Path relative to the policy root, e.g. "logical_branching.yaml"
    /// or "policies/context_memory_2025-11-12T22-43.json".
    pub relative_path: String,

    /// Expected SHA-256 of the file contents, as lowercase hex string.
    pub sha256: String,
}

/// Snapshot metadata stored inside each snapshot directory.
///
/// Convention:
///   snapshots_root/<snapshot_id>/snapshot_meta.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotMeta {
    /// Logical snapshot identifier (also directory name).
    pub id: String,

    /// Time when snapshot was captured.
    pub timestamp: DateTime<Utc>,

    /// Convenience pointers for the primary policy artifacts.
    pub branching_policy: String,
    pub plugin_graph: String,
    pub memory_snapshot: String,

    /// Optional execution limiter file (Python side control).
    pub execution_limiter: Option<String>,

    /// Full list of tracked files and their hashes.
    pub files: Vec<SnapshotFile>,
}

/// A lightweight listing entry for `list_snapshots`.
#[derive(Debug, Clone)]
pub struct SnapshotSummary {
    pub id: String,
    pub timestamp: DateTime<Utc>,
}

/// A simple log entry to append to an audit / rollback log file.
///
/// This is independent of the Phase III `policy_audit_log` crate,
/// but can be tailed or ingested into it later.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackLogEntry {
    pub timestamp: DateTime<Utc>,
    pub operator: String,
    pub snapshot_id: String,
    pub reason: Option<String>,
    pub success: bool,
    pub details: String,
}

impl SnapshotMeta {
    /// Compute a `SnapshotSummary` view.
    pub fn summary(&self) -> SnapshotSummary {
        SnapshotSummary {
            id: self.id.clone(),
            timestamp: self.timestamp,
        }
    }

    /// Resolve a relative path in this snapshot to a concrete file
    /// inside `snapshot_dir`.
    pub fn resolve_path(&self, snapshot_dir: &PathBuf, relative: &str) -> PathBuf {
        snapshot_dir.join(relative)
    }
}