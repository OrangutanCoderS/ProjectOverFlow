use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SysBlockRequest {
    pub pid: i32,
    pub syscall: String,
    pub mode: String, // "log" | "enforce"
}

impl SysBlockRequest {
    pub fn validate(&self) -> Result<(), String> {
        if self.pid <= 0 {
            return Err("Invalid PID".into());
        }
        if self.syscall.trim().is_empty() {
            return Err("Syscall name cannot be empty".into());
        }
        Ok(())
    }
}