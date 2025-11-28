use chrono::Utc;
use serde::Serialize;
use std::fs::{create_dir_all, OpenOptions};
use std::io::Write;
use std::path::Path;

#[derive(Serialize)]
struct SyscallAuditEvent<'a> {
    timestamp: String,
    pid: i32,
    syscall: &'a str,
    action: &'a str,
    platform: &'a str,
    result: &'a str,
}

pub fn log_event(pid: i32, syscall: &str, action: &str, platform: &str, result: &str) {
    let log_path = Path::new("crates/syscall_blocker/logs/system_log.json");

    if let Err(e) = create_dir_all(log_path.parent().unwrap_or(Path::new("."))) {
        eprintln!("[audit] log dir create failed: {}", e);
        return;
    }

    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(log_path) {
        let event = SyscallAuditEvent {
            timestamp: Utc::now().to_rfc3339(),
            pid,
            syscall,
            action,
            platform,
            result,
        };
        if let Ok(line) = serde_json::to_string(&event) {
            let _ = writeln!(file, "{}", line);
        }
    }
}