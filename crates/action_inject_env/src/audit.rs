use chrono::Utc;
use serde::Serialize;
use std::fs::{create_dir_all, OpenOptions};
use std::io::Write;
use std::path::Path;

/// A single audit event stored as newline-delimited JSON.
#[derive(Debug, Serialize)]
struct EnvInjectAudit<'a> {
    ts_utc: String,
    program: &'a str,
    pid: Option<i32>,
    mode: &'a str,
    /// We only store the keys for privacy; values are redacted by design.
    env_keys: Vec<&'a str>,
    inherit: bool,
    context: &'a str,
}

/// Write an audit line into `logs/system_log.json` (re-uses top-level logs dir).
pub fn write_audit_log<'a, I>(
    program: &str,
    pid: Option<i32>,
    mode: &str,
    env_keys: I,
    inherit: bool,
    context: &str,
) -> Result<(), String>
where
    I: Iterator<Item = &'a str>,
{
    let logs_dir = Path::new("logs");
    if let Err(e) = create_dir_all(logs_dir) {
        return Err(format!("create logs dir: {e}"));
    }

    let file_path = logs_dir.join("system_log.json");
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&file_path)
        .map_err(|e| format!("open audit file {file_path:?}: {e}"))?;

    let event = EnvInjectAudit {
        ts_utc: Utc::now().to_rfc3339(),
        program,
        pid,
        mode,
        env_keys: env_keys.collect(),
        inherit,
        context,
    };

    let line = serde_json::to_string(&event).map_err(|e| format!("serialize: {e}"))?;
    file.write_all(line.as_bytes())
        .and_then(|_| file.write_all(b"\n"))
        .map_err(|e| format!("write audit: {e}"))?;

    Ok(())
}