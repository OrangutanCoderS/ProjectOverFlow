use chrono::Utc;
use serde::Serialize;
use std::{
    fs::{create_dir_all, OpenOptions},
    io::Write,
    path::Path,
};

#[derive(Debug, Serialize)]
struct MirrorAudit<'a> {
    ts_utc: String,
    original_pid: i32,
    mirror_pid: Option<u32>,
    sandboxed: bool,
    context: &'a str,
    result: &'a str,
}

pub fn write_audit_log<'a>(
    original_pid: i32,
    mirror_pid: Option<u32>,
    sandboxed: bool,
    context: &'a str,
    result: &'a str,
) -> Result<(), String> {
    let logs_dir = Path::new("logs");
    create_dir_all(logs_dir).map_err(|e| e.to_string())?;
    let path = logs_dir.join("mirror_process_log.json");
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|e| format!("open log {path:?}: {e}"))?;
    let record = MirrorAudit {
        ts_utc: Utc::now().to_rfc3339(),
        original_pid,
        mirror_pid,
        sandboxed,
        context,
        result,
    };
    let line = serde_json::to_string(&record).map_err(|e| e.to_string())?;
    file.write_all(line.as_bytes()).and_then(|_| file.write_all(b"\n")).map_err(|e| e.to_string())
}