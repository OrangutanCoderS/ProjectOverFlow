use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use chrono::Utc;
use sha2::{Digest, Sha256};

use crate::error::RollbackError;
use crate::model::{RollbackLogEntry, SnapshotFile, SnapshotMeta, SnapshotSummary};

const META_FILE_NAME: &str = "snapshot_meta.json";
const LOCK_FILE_NAME: &str = ".rollback.lock";

/// Plan produced by `simulate_revert` – which files will be overwritten.
#[derive(Debug, Clone)]
pub struct RollbackPlan {
    pub snapshot_id: String,
    pub files_to_replace: Vec<String>,
}

/// Engine responsible for listing, validating, and applying rollbacks.
#[derive(Debug, Clone)]
pub struct RollbackEngine {
    snapshot_root: PathBuf,
    policy_root: PathBuf,
    audit_log_path: Option<PathBuf>,
}

impl RollbackEngine {
    /// Create a new engine instance.
    ///
    /// - `snapshot_root`: directory containing per-snapshot subdirs.
    /// - `policy_root`: directory containing live policy files.
    /// - `audit_log_path`: optional JSONL log file for rollback events.
    pub fn new(
        snapshot_root: PathBuf,
        policy_root: PathBuf,
        audit_log_path: Option<PathBuf>,
    ) -> Self {
        Self {
            snapshot_root,
            policy_root,
            audit_log_path,
        }
    }

    /// Convenience accessors (useful in tests or callers).
    pub fn snapshot_root(&self) -> &Path {
        &self.snapshot_root
    }

    pub fn policy_root(&self) -> &Path {
        &self.policy_root
    }

    /// Enumerate all snapshots that have a readable meta file.
    pub fn list_snapshots(&self) -> Result<Vec<SnapshotSummary>, RollbackError> {
        let mut out = Vec::new();

        if !self.snapshot_root.exists() {
            // No snapshots yet is not an error.
            return Ok(out);
        }

        for entry in fs::read_dir(&self.snapshot_root)? {
            let entry = entry?;
            if !entry.file_type()?.is_dir() {
                continue;
            }

            let id = match entry.file_name().into_string() {
                Ok(s) => s,
                Err(_) => continue,
            };

            if let Ok(meta) = self.load_meta(&id) {
                out.push(meta.summary());
            }
        }

        // Sort snapshots by timestamp ascending for deterministic output.
        out.sort_by_key(|s| s.timestamp);
        Ok(out)
    }

    /// Validate the integrity of a given snapshot.
    ///
    /// - Ensures meta file exists and is parseable.
    /// - Ensures every declared file exists.
    /// - Ensures SHA-256 of each file matches the recorded value.
    pub fn validate_integrity(&self, snapshot_id: &str) -> Result<(), RollbackError> {
        let meta = self.load_meta(snapshot_id)?;
        let snapshot_dir = self.snapshot_dir(snapshot_id);

        for file in &meta.files {
            let path = snapshot_dir.join(&file.relative_path);
            let actual = self.compute_file_sha256(&path)?;

            if !constant_time_eq(&actual, &file.sha256) {
                return Err(RollbackError::IntegrityFailure {
                    snapshot_id: snapshot_id.to_string(),
                    path,
                });
            }
        }

        Ok(())
    }

    /// Produce a plan describing which files will be replaced for a rollback.
    pub fn simulate_revert(&self, snapshot_id: &str) -> Result<RollbackPlan, RollbackError> {
        let meta = self.load_meta(snapshot_id)?;

        let files_to_replace = meta.files.iter().map(|f| f.relative_path.clone()).collect();

        Ok(RollbackPlan {
            snapshot_id: snapshot_id.to_string(),
            files_to_replace,
        })
    }

    /// Perform the rollback:
    ///
    /// - Acquire a global lock (simple lock file in `snapshot_root`).
    /// - Validate snapshot integrity.
    /// - Backup current live policy files into a `live_backup_*` snapshot.
    /// - Copy snapshot files into `policy_root`.
    /// - Append rollback record to audit log (if configured).
    pub fn rollback_to(
        &self,
        snapshot_id: &str,
        operator: &str,
        reason: Option<&str>,
    ) -> Result<(), RollbackError> {
        self.ensure_policy_root()?;

        let lock_path = self.snapshot_root.join(LOCK_FILE_NAME);
        let _lock_guard = self.acquire_lock(&lock_path)?;

        // Validate snapshot integrity first
        self.validate_integrity(snapshot_id)?;

        let meta = self.load_meta(snapshot_id)?;
        let snapshot_dir = self.snapshot_dir(snapshot_id);

        // Best-effort live backup of current files
        self.capture_live_backup(&meta.files)?;

        // Apply snapshot onto policy root
        for file in &meta.files {
            let src = snapshot_dir.join(&file.relative_path);
            let dst = self.policy_root.join(&file.relative_path);
            copy_file(&src, &dst)?;
        }

        // Log rollback event (if configured)
        let entry = RollbackLogEntry {
            timestamp: Utc::now(),
            operator: operator.to_string(),
            snapshot_id: snapshot_id.to_string(),
            reason: reason.map(|s| s.to_string()),
            success: true,
            details: format!(
                "Rollback applied from snapshot `{}` affecting {} files",
                snapshot_id,
                meta.files.len()
            ),
        };

        self.append_audit(&entry)?;

        Ok(())
    }

    /// Ensure that policy root exists and is a directory.
    fn ensure_policy_root(&self) -> Result<(), RollbackError> {
        if self.policy_root.is_dir() {
            Ok(())
        } else {
            Err(RollbackError::InvalidPolicyRoot(
                self.policy_root.clone(),
            ))
        }
    }

    fn snapshot_dir(&self, snapshot_id: &str) -> PathBuf {
        self.snapshot_root.join(snapshot_id)
    }

    fn meta_path(&self, snapshot_id: &str) -> PathBuf {
        self.snapshot_dir(snapshot_id).join(META_FILE_NAME)
    }

    fn load_meta(&self, snapshot_id: &str) -> Result<SnapshotMeta, RollbackError> {
        let path = self.meta_path(snapshot_id);

        if !path.exists() {
            return Err(RollbackError::SnapshotNotFound(snapshot_id.to_string()));
        }

        let data = fs::read_to_string(&path)?;
        let meta: SnapshotMeta = serde_json::from_str(&data)?;

        if meta.id != snapshot_id {
            return Err(RollbackError::InvalidSnapshot(snapshot_id.to_string()));
        }

        Ok(meta)
    }

    fn compute_file_sha256(&self, path: &Path) -> Result<String, RollbackError> {
        let mut hasher = Sha256::new();
        let bytes = fs::read(path)?;
        hasher.update(&bytes);
        let digest = hasher.finalize();
        Ok(hex_encode(&digest))
    }

    /// Capture a simple backup of current live files using the declared file set
    /// from a snapshot. This does not generate full metadata; it is best effort.
    fn capture_live_backup(&self, files: &[SnapshotFile]) -> Result<(), RollbackError> {
        if files.is_empty() {
            return Ok(());
        }

        let backup_id = format!("live_backup_{}", Utc::now().format("%Y-%m-%dT%H-%M-%S"));
        let backup_dir = self.snapshot_root.join(&backup_id);
        fs::create_dir_all(&backup_dir)?;

        for file in files {
            let src = self.policy_root.join(&file.relative_path);
            if src.exists() {
                let dst = backup_dir.join(&file.relative_path);
                if let Some(parent) = dst.parent() {
                    fs::create_dir_all(parent)?;
                }
                // If this fails, we still want the rollback to fail loudly.
                copy_file(&src, &dst)?;
            }
        }

        Ok(())
    }

    /// Append a rollback entry to the configured audit log (JSONL format).
    fn append_audit(&self, entry: &RollbackLogEntry) -> Result<(), RollbackError> {
        let path = match &self.audit_log_path {
            Some(p) => p,
            None => return Ok(()),
        };

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;

        let json = serde_json::to_string(entry)?;
        file.write_all(json.as_bytes())?;
        file.write_all(b"\n")?;
        Ok(())
    }

    /// Acquire a simple global lock by creating a lock file.
    fn acquire_lock(&self, lock_path: &Path) -> Result<RollbackLockGuard, RollbackError> {
        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(lock_path)
        {
            Ok(_) => Ok(RollbackLockGuard {
                path: lock_path.to_path_buf(),
            }),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                Err(RollbackError::AlreadyLocked(lock_path.to_path_buf()))
            }
            Err(e) => Err(RollbackError::Io(e)),
        }
    }
}

/// RAII guard for the rollback lock.
struct RollbackLockGuard {
    path: PathBuf,
}

impl Drop for RollbackLockGuard {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

/// Copy a single file, creating parent directories if needed.
fn copy_file(src: &Path, dst: &Path) -> Result<(), RollbackError> {
    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(src, dst)?;
    Ok(())
}

/// Constant-time-ish equality for hex strings with same length.
/// This is not crypto-grade, but avoids obviously early-exit branches.
fn constant_time_eq(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let mut diff = 0u8;
    for (x, y) in a.bytes().zip(b.bytes()) {
        diff |= x ^ y;
    }
    diff == 0
}

/// Minimal hex encoder to avoid pulling extra crates.
fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 0x0f) as usize] as char);
    }
    out
}