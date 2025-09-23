use crate::model::InterventionResult;
use crate::errors::ExecutionError;
use crate::util::ensure_logs_dir;

use std::fs::{OpenOptions};
use std::io::{Write};
use std::path::PathBuf;
use anyhow::Context;

/// JSONL logger interface
pub trait InterventionLogger: Send + Sync + 'static {
    /// Append result; if `unify` true, also mirror to plugin_log.json.
    fn append(&self, res: &InterventionResult, unify: bool) -> anyhow::Result<()>;
}

#[derive(Clone, Debug)]
pub struct LoggerTarget {
    pub intervention_log: PathBuf,
    pub plugin_log: PathBuf,
}

impl Default for LoggerTarget {
    fn default() -> Self {
        Self {
            intervention_log: PathBuf::from("logs/intervention_log.json"),
            plugin_log: PathBuf::from("logs/plugin_log.json"),
        }
    }
}

#[derive(Clone)]
pub struct JsonlLogger {
    target: LoggerTarget,
}

impl JsonlLogger {
    pub fn new_default() -> anyhow::Result<Self> {
        ensure_logs_dir()?;
        Ok(Self { target: LoggerTarget::default() })
    }

    fn append_one(path: &PathBuf, line: &str) -> anyhow::Result<()> {
        let mut f = OpenOptions::new()
            .create(true).append(true)
            .open(path)
            .with_context(|| format!("open log path {}", path.display()))?;
        f.write_all(line.as_bytes())
            .and_then(|_| f.write_all(b"\n"))
            .with_context(|| "write JSONL")?;
        Ok(())
    }
}

impl InterventionLogger for JsonlLogger {
    fn append(&self, res: &InterventionResult, unify: bool) -> anyhow::Result<()> {
        let js = serde_json::to_string(res)?;
        Self::append_one(&self.target.intervention_log, &js)?;
        if unify {
            Self::append_one(&self.target.plugin_log, &js)?;
        }
        Ok(())
    }
}
