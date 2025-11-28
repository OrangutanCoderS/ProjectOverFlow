use std::{fs::OpenOptions, io::Write, sync::Mutex};
use crate::request::KillRequest;
use crate::errors::ManagerError;
use chrono::Utc;
use serde_json::json;

pub struct AuditLogger {
    file: Mutex<std::fs::File>,
}

impl AuditLogger {
    pub fn new(path: &str) -> Result<Self, ManagerError> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
        Ok(Self { file: Mutex::new(file) })
    }

    pub fn log_simulated(&self, req: &KillRequest) -> Result<(), ManagerError> {
        let entry = json!({
            "time": Utc::now(),
            "request_id": req.id,
            "pid": req.target_pid,
            "action": format!("{:?}", req.mode),
            "simulated": true
        });
        self.write_entry(entry)
    }

    pub fn log_result(&self, req: &KillRequest, ok: bool, dur: u128) -> Result<(), ManagerError> {
        let entry = json!({
            "time": Utc::now(),
            "request_id": req.id,
            "pid": req.target_pid,
            "action": format!("{:?}", req.mode),
            "success": ok,
            "duration_ms": dur
        });
        self.write_entry(entry)
    }

    fn write_entry(&self, entry: serde_json::Value) -> Result<(), ManagerError> {
        let mut f = self.file.lock().unwrap();
        writeln!(f, "{}", entry)?;
        Ok(())
    }
}