use chrono::Utc;
use serde::Serialize;
use std::fs::{create_dir_all, OpenOptions};
use std::io::Write;
use std::path::Path;

#[derive(Debug, Serialize)]
struct ThrottleAudit<'a> {
    ts_utc: String,
    pid: i32,
    mode: &'a str,
    intensity: f32,
    duration_secs: Option<u64>,
    restored: bool,
    context: &'a str,
}

pub fn write_audit_log(
    pid: i32,
    mode: &str,
    intensity: f32,
    duration_secs: Option<u64>,
    restored: bool,
    context: &str,
) -> Result<(), String> {
    let logs_dir = Path::new("logs");
    create_dir_all(logs_dir).map_err(|e| format!("create logs dir: {e}"))?;
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(logs_dir.join("system_log.json"))
        .map_err(|e| format!("open log: {e}"))?;

    let event = ThrottleAudit {
        ts_utc: Utc::now().to_rfc3339(),
        pid,
        mode,
        intensity,
        duration_secs,
        restored,
        context,
    };
    let line = serde_json::to_string(&event).map_err(|e| format!("serialize: {e}"))?;
    file.write_all(line.as_bytes())
        .and_then(|_| file.write_all(b"\n"))
        .map_err(|e| format!("write audit: {e}"))?;
    Ok(())
}