/*!
 Module 16 — Crash Detector (crashdet)
 Goal: Detect application crashes and kernel panics by polling system crash logs.
*/

use anyhow::{anyhow, bail, Context, Result};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::{Duration, SystemTime};
use thiserror::Error;

// overflow-utils
use overflow_utils::utc_iso8601;

/* ============================
   Data Model
   ============================ */

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CrashInfo {
    pub timestamp: String,             // ISO-8601 UTC (observation time)
    pub pid: Option<i32>,              // Process ID if available
    pub process: String,               // Executable / bundle name
    pub crash_reason: String,          // EXC_BAD_ACCESS, SIGSEGV, Termination: ...
    pub uptime_at_crash_sec: Option<u64>,
    pub severity: String,              // "app_crash" | "panic"
}

impl CrashInfo {
    pub fn validate(&self) -> Result<()> {
        if self.timestamp.is_empty() { bail!("timestamp empty"); }
        if self.process.trim().is_empty() { bail!("process empty"); }
        if self.crash_reason.trim().is_empty() { bail!("crash_reason empty"); }
        if !(self.severity == "app_crash" || self.severity == "panic") {
            bail!("invalid severity");
        }
        Ok(())
    }
}

/* ============================
   Config
   ============================ */

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrashConfig {
    pub poll_interval_ms: u64,
    pub max_entries_per_sample: usize,
    pub max_file_bytes: usize,
    pub crash_dirs: Vec<PathBuf>,
}

impl Default for CrashConfig {
    fn default() -> Self {
        Self {
            poll_interval_ms: 5_000,
            max_entries_per_sample: 100,
            max_file_bytes: 1_048_576, // 1 MiB
            crash_dirs: vec![],
        }
    }
}

/* ============================
   Public API
   ============================ */

pub fn poll_crashes(cfg: &CrashConfig) -> Result<Vec<CrashInfo>> {
    BACKEND.sample(cfg)
}

pub fn spawn_crash_watcher(cfg: CrashConfig) -> Receiver<CrashInfo> {
    let (tx, rx): (Sender<CrashInfo>, Receiver<CrashInfo>) = mpsc::channel();
    std::thread::spawn(move || {
        let interval = Duration::from_millis(cfg.poll_interval_ms.max(100));
        loop {
            match BACKEND.sample(&cfg) {
                Ok(list) => {
                    for mut ev in list {
                        ev.timestamp = utc_iso8601();
                        let _ = tx.send(ev);
                    }
                }
                Err(_) => { /* fail-closed */ }
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
    fn sample(&self, cfg: &CrashConfig) -> Result<Vec<CrashInfo>>;
}

static BACKEND: Lazy<Box<dyn Backend>> = Lazy::new(|| {
    if MacCrashBackend::available() {
        Box::new(MacCrashBackend)
    } else if LinuxCrashBackend::available() {
        Box::new(LinuxCrashBackend)
    } else if WindowsCrashBackend::available() {
        Box::new(WindowsCrashBackend)
    } else {
        Box::new(NullBackend)
    }
});

/* ============================
   macOS Implementation
   ============================ */

struct MacCrashBackend;

impl MacCrashBackend {
    fn available() -> bool { cfg!(target_os = "macos") }

    fn default_crash_paths() -> Vec<PathBuf> {
        let mut v = Vec::new();
        if let Some(home) = std::env::var_os("HOME") {
            v.push(PathBuf::from(home).join("Library/Logs/DiagnosticReports"));
        }
        v.push(PathBuf::from("/Library/Logs/DiagnosticReports"));
        v.push(PathBuf::from("/Library/Logs/panic.log"));
        v
    }

    fn is_crash_like(path: &Path) -> bool {
        if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
            name.ends_with(".crash") || name.ends_with(".panic") || name == "panic.log"
        } else { false }
    }

    fn read_file_capped(path: &Path, max_bytes: usize) -> Result<String> {
        let meta = fs::metadata(path).with_context(|| format!("metadata {}", path.display()))?;
        if !meta.is_file() {
            bail!("not a file");
        }
        let to_read = meta.len().min(max_bytes as u64) as usize;
        let mut f = fs::File::open(path).with_context(|| format!("open {}", path.display()))?;
        let mut buf = vec![0u8; to_read];
        let n = f.read(&mut buf).with_context(|| format!("read {}", path.display()))?;
        buf.truncate(n);
        Ok(String::from_utf8_lossy(&buf).to_string())
    }

    fn parse_crash_file(path: &Path, max_bytes: usize) -> Option<CrashInfo> {
        let s = Self::read_file_capped(path, max_bytes).ok()?;

        static RE_PROC: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r#"(?m)^\s*Process:\s*(?P<name>.+?)\s*\[(?P<pid>\d+)\]\s*$"#).unwrap()
        });
        static RE_REASON: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r#"(?m)^\s*(Exception Type|Termination Reason):\s*(?P<reason>.+?)\s*$"#).unwrap()
        });

        let caps = RE_PROC.captures(&s)?;
        let process = caps.name("name")?.as_str().trim().to_string();
        let pid = caps.name("pid").and_then(|m| m.as_str().parse::<i32>().ok());
        let reason = RE_REASON
            .captures(&s)
            .and_then(|c| c.name("reason").map(|m| m.as_str().trim().to_string()))
            .unwrap_or_else(|| "Unknown".to_string());

        let info = CrashInfo {
            timestamp: utc_iso8601(),
            pid,
            process,
            crash_reason: reason,
            uptime_at_crash_sec: None,
            severity: "app_crash".to_string(),
        };
        info.validate().ok()?;
        Some(info)
    }

    fn parse_kernel_panic(path: &Path, max_bytes: usize) -> Option<CrashInfo> {
        let s = Self::read_file_capped(path, max_bytes).ok()?;
        let reason = s
            .lines()
            .find(|l| l.contains("panic(") || l.contains("*** Panic Report ***") || l.contains("panic:"))
            .map(|l| l.trim().to_string())
            .unwrap_or_else(|| "kernel panic detected".to_string());

        let info = CrashInfo {
            timestamp: utc_iso8601(),
            pid: None,
            process: "kernel".to_string(),
            crash_reason: reason,
            uptime_at_crash_sec: None,
            severity: "panic".to_string(),
        };
        info.validate().ok()?;
        Some(info)
    }

    fn collect_candidates(dirs: &[PathBuf], cap: usize) -> Vec<PathBuf> {
        let mut files: Vec<(SystemTime, PathBuf)> = Vec::new();

        for d in dirs {
            if d.is_file() {
                if Self::is_crash_like(d) {
                    if let Ok(m) = fs::metadata(d) {
                        if m.is_file() {
                            files.push((m.modified().unwrap_or(SystemTime::UNIX_EPOCH), d.clone()));
                        }
                    }
                }
                continue;
            }
            let rd = match fs::read_dir(d) { Ok(r) => r, Err(_) => continue };
            for ent in rd.flatten() {
                let p = ent.path();
                if !Self::is_crash_like(&p) { continue; }
                if let Ok(m) = ent.metadata() {
                    if m.is_file() {
                        files.push((m.modified().unwrap_or(SystemTime::UNIX_EPOCH), p));
                    }
                }
            }
        }

        files.sort_by(|a, b| b.0.cmp(&a.0));
        files.into_iter().take(cap).map(|(_, p)| p).collect()
    }
}

impl Backend for MacCrashBackend {
    fn sample(&self, cfg: &CrashConfig) -> Result<Vec<CrashInfo>> {
        let dirs = if cfg.crash_dirs.is_empty() {
            Self::default_crash_paths()
        } else {
            cfg.crash_dirs.clone()
        };
        let candidates = Self::collect_candidates(&dirs, cfg.max_entries_per_sample);
        let mut out = Vec::new();
        for path in candidates {
            let is_panic = path
                .file_name()
                .and_then(|s| s.to_str())
                .map(|f| f.ends_with(".panic") || f == "panic.log")
                .unwrap_or(false);
            let ev = if is_panic {
                Self::parse_kernel_panic(&path, cfg.max_file_bytes)
            } else {
                Self::parse_crash_file(&path, cfg.max_file_bytes)
            };
            if let Some(info) = ev {
                out.push(info);
            }
        }
        Ok(out)
    }
}

/* ============================ */ 
// Fallbacks
struct LinuxCrashBackend;
impl LinuxCrashBackend { fn available() -> bool { cfg!(target_os = "linux") } }
impl Backend for LinuxCrashBackend {
    fn sample(&self, _cfg: &CrashConfig) -> Result<Vec<CrashInfo>> { Ok(vec![]) }
}

struct WindowsCrashBackend;
impl WindowsCrashBackend { fn available() -> bool { cfg!(target_os = "windows") } }
impl Backend for WindowsCrashBackend {
    fn sample(&self, _cfg: &CrashConfig) -> Result<Vec<CrashInfo>> { Ok(vec![]) }
}

struct NullBackend;
impl Backend for NullBackend {
    fn sample(&self, _cfg: &CrashConfig) -> Result<Vec<CrashInfo>> { Ok(vec![]) }
}

/* ============================ */
#[derive(Debug, Error)]
pub enum CrashError {
    #[error("backend unavailable")]
    BackendUnavailable,
    #[error("parse error: {0}")]
    Parse(String),
}