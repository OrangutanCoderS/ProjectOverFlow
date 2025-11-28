use chrono::Utc;
use serde_json::json;
use std::{fs::OpenOptions, io::Write, path::Path};
use crate::errors::ManagerError;

pub fn write_audit_log(pid: i32, mode: &str, context: &str) -> Result<(), ManagerError> {
    let log_path = Path::new("logs/action_log.json");

    let entry = json!({
        "timestamp": Utc::now().to_rfc3339(),
        "pid": pid,
        "action": mode,
        "context": context
    });

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)
        .map_err(|e| ManagerError::AuditWriteError(e.to_string()))?;

    writeln!(file, "{}", entry.to_string())
        .map_err(|e| ManagerError::AuditWriteError(e.to_string()))?;

    Ok(())
}