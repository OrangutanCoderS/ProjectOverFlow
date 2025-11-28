use thiserror::Error;

#[derive(Debug, Error)]
pub enum MirrorError {
    #[error("invalid pid: {0}")]
    InvalidPid(i32),
    #[error("process snapshot failed: {0}")]
    SnapshotFailed(String),
    #[error("sandbox isolation failed: {0}")]
    SandboxFailed(String),
    #[error("spawn failed: {0}")]
    SpawnFailed(String),
    #[error("audit log failed: {0}")]
    Audit(String),
}