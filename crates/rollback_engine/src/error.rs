use std::io;
use std::path::PathBuf;
use thiserror::Error;

/// Errors emitted by the rollback engine.
#[derive(Debug, Error)]
pub enum RollbackError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("snapshot `{0}` not found")]
    SnapshotNotFound(String),

    #[error("snapshot `{0}` has invalid or missing metadata")]
    InvalidSnapshot(String),

    #[error(
        "integrity check failed for snapshot `{snapshot_id}` at path `{path:?}`"
    )]
    IntegrityFailure {
        snapshot_id: String,
        path: PathBuf,
    },

    #[error("rollback already in progress (lock file present at {0:?})")]
    AlreadyLocked(PathBuf),

    #[error("policy root `{0:?}` is not a directory")]
    InvalidPolicyRoot(PathBuf),
}