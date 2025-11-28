use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KillMode {
    SoftKill,
    HardKill,
    Suspend,
    Resume,
    Throttle,
    Isolate,
    Purge,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KillRequest {
    pub id: Uuid,
    pub target_pid: i32,
    pub mode: KillMode,
    pub requested_by: String,
    pub dry_run: bool,
    pub policy_id: Option<String>,
    pub timestamp: DateTime<Utc>,
}

impl KillRequest {
    pub fn new(pid: i32, mode: KillMode, by: &str, dry_run: bool) -> Self {
        Self {
            id: Uuid::new_v4(),
            target_pid: pid,
            mode,
            requested_by: by.to_string(),
            dry_run,
            policy_id: None,
            timestamp: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KillResult {
    pub request_id: Uuid,
    pub success: bool,
    pub message: String,
    pub duration_ms: u128,
}