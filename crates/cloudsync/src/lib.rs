/*!
 Module 17 — Cloud Sync Detector (cloudsync)
 Goal: Detect cloud sync client activity (Dropbox, OneDrive, iCloud, Google Drive).
 Phase I: process-name + file-activity heuristics.

 Design:
 - Backend trait with platform-specific impls.
 - Public API: one-shot polling + background watcher.
 - Test builds use FakeBackend for deterministic results.
*/

use anyhow::Result;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::mpsc::{self, Receiver};
use thiserror::Error;

// overflow-utils
use overflow_utils::utc_iso8601;

/* ============================
   Data Model
   ============================ */

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CloudSyncEvent {
    pub timestamp: String,          // ISO-8601 UTC
    pub provider: String,           // dropbox | onedrive | icloud | gdrive
    pub burst_score: f64,           // normalized 0..1 heuristic
    pub process: Option<String>,    // client name
}

impl CloudSyncEvent {
    pub fn validate(&self) -> Result<()> {
        if self.timestamp.is_empty() {
            anyhow::bail!("timestamp empty");
        }
        if self.provider.is_empty() {
            anyhow::bail!("provider empty");
        }
        if !(0.0..=1.0).contains(&self.burst_score) {
            anyhow::bail!("burst_score out of range");
        }
        Ok(())
    }
}

/* ============================
   Config
   ============================ */

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudSyncConfig {
    pub poll_interval_ms: u64,
    pub max_entries_per_sample: usize,
}

impl Default for CloudSyncConfig {
    fn default() -> Self {
        Self {
            poll_interval_ms: 2000,
            max_entries_per_sample: 512,
        }
    }
}

/* ============================
   Public API
   ============================ */

pub fn poll_cloudsync(cfg: &CloudSyncConfig) -> Result<Vec<CloudSyncEvent>> {
    BACKEND.sample(cfg)
}

pub fn spawn_cloudsync_watcher(cfg: CloudSyncConfig) -> Receiver<CloudSyncEvent> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let interval = std::time::Duration::from_millis(cfg.poll_interval_ms.max(100));
        loop {
            if let Ok(list) = BACKEND.sample(&cfg) {
                for mut ev in list {
                    ev.timestamp = utc_iso8601();
                    let _ = tx.send(ev);
                }
            }
            std::thread::sleep(interval);
        }
    });
    rx
}

/* ============================
   Backend trait + selection
   ============================ */

trait Backend: Send + Sync {
    fn name(&self) -> &'static str;
    fn sample(&self, cfg: &CloudSyncConfig) -> Result<Vec<CloudSyncEvent>>;
}

#[cfg(not(test))]
static BACKEND: Lazy<Box<dyn Backend>> = Lazy::new(|| {
    if MacBackend::available() {
        Box::new(MacBackend) as Box<dyn Backend>
    } else if LinuxBackend::available() {
        Box::new(LinuxBackend) as Box<dyn Backend>
    } else if WindowsBackend::available() {
        Box::new(WindowsBackend) as Box<dyn Backend>
    } else {
        Box::new(NullBackend) as Box<dyn Backend>
    }
});

#[cfg(test)]
static BACKEND: Lazy<Box<dyn Backend>> = Lazy::new(|| {
    Box::new(FakeBackend) as Box<dyn Backend>
});

/* ============================
   macOS / Linux / Windows impls (stubs Phase I)
   ============================ */

struct MacBackend;
impl MacBackend {
    fn available() -> bool { cfg!(target_os = "macos") }
}
impl Backend for MacBackend {
    fn name(&self) -> &'static str { "macos_cloudsync" }
    fn sample(&self, _cfg: &CloudSyncConfig) -> Result<Vec<CloudSyncEvent>> {
        Ok(vec![]) // TODO Phase II: real heuristics
    }
}

struct LinuxBackend;
impl LinuxBackend { fn available() -> bool { cfg!(target_os = "linux") } }
impl Backend for LinuxBackend {
    fn name(&self) -> &'static str { "linux_cloudsync" }
    fn sample(&self, _cfg: &CloudSyncConfig) -> Result<Vec<CloudSyncEvent>> {
        Ok(vec![])
    }
}

struct WindowsBackend;
impl WindowsBackend { fn available() -> bool { cfg!(target_os = "windows") } }
impl Backend for WindowsBackend {
    fn name(&self) -> &'static str { "windows_cloudsync" }
    fn sample(&self, _cfg: &CloudSyncConfig) -> Result<Vec<CloudSyncEvent>> {
        Ok(vec![])
    }
}

struct NullBackend;
impl Backend for NullBackend {
    fn name(&self) -> &'static str { "null" }
    fn sample(&self, _cfg: &CloudSyncConfig) -> Result<Vec<CloudSyncEvent>> {
        Ok(vec![])
    }
}

/* ============================
   Test-only Fake Backend
   ============================ */

#[cfg(test)]
struct FakeBackend;

#[cfg(test)]
impl Backend for FakeBackend {
    fn name(&self) -> &'static str { "fake_cloudsync" }
    fn sample(&self, _cfg: &CloudSyncConfig) -> Result<Vec<CloudSyncEvent>> {
        Ok(vec![CloudSyncEvent {
            timestamp: utc_iso8601(),
            provider: "dropbox".to_string(),
            burst_score: 0.8,
            process: Some("Dropbox".to_string()),
        }])
    }
}

/* ============================
   Error surface
   ============================ */

#[derive(Debug, Error)]
pub enum CloudSyncError {
    #[error("backend unavailable")]
    BackendUnavailable,
    #[error("parse error: {0}")]
    Parse(String),
}