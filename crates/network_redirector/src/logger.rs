use crate::RedirectLogEntry;
use chrono::Utc;
use std::fs::{OpenOptions};
use std::io::{Write, Result};

pub struct JsonlLogger {
    path: String,
}

impl JsonlLogger {
    pub fn new(path: &str) -> Self {
        Self { path: path.to_string() }
    }

    pub fn log(&self, entry: &RedirectLogEntry) -> Result<()> {
        let mut f = OpenOptions::new().create(true).append(true).open(&self.path)?;
        let line = serde_json::to_string(entry).unwrap();
        writeln!(f, "{}", line)?;
        f.sync_all()?; // durability
        Ok(())
    }

    pub fn log_simple(&self, stage: &str, req: crate::RedirectRequest, res: Option<String>, err: Option<String>) {
        let entry = RedirectLogEntry {
            timestamp: Utc::now(),
            stage: stage.into(),
            request: req,
            rule: None,
            backend_result: res,
            error: err,
        };
        let _ = self.log(&entry);
    }
}