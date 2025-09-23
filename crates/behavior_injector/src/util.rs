use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use std::fs;
use std::path::Path;

pub fn utc_timestamp() -> String {
    OffsetDateTime::now_utc().format(&Rfc3339).unwrap_or_else(|_| "1970-01-01T00:00:00Z".into())
}

pub fn ensure_logs_dir() -> anyhow::Result<()> {
    let p = Path::new("logs");
    if !p.exists() { fs::create_dir_all(p)?; }
    Ok(())
}

/// Self-protection: never act against daemon / critical PIDs.
/// Extend with allowlists from configs if needed.
#[derive(Clone, Debug)]
pub struct ProtectedPidPolicy {
    pub protected: Vec<i32>,
}

impl Default for ProtectedPidPolicy {
    fn default() -> Self {
        // 0, 1 are typical reserved; daemon pid to be injected by caller if known.
        Self { protected: vec![0, 1] }
    }
}

impl ProtectedPidPolicy {
    pub fn is_protected(&self, pid: i32) -> bool {
        self.protected.contains(&pid)
    }
}
