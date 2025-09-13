//! Sandbox Manager (Module 20)
//! Provides a safe abstraction for launching processes inside a sandbox profile (macOS only).
//! On Linux/Windows, stubs return events marking sandboxing as unsupported.

use anyhow::Result;
use chrono::{DateTime, Utc};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use thiserror::Error;
use std::sync::Mutex;

/// Log file for sandbox events
static LOG_PATH: &str = "logs/sandbox_log.json";

/// Global lock to avoid concurrent log writes
static LOG_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

/// Errors specific to sandbox operations
#[derive(Debug, Error)]
pub enum SandboxError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("Sandbox execution failed: {0}")]
    ExecFailure(String),

    #[error("Unsupported on this platform")]
    Unsupported,
}

/// Sandbox configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    /// Path to sandbox profile (macOS `.sb` file)
    pub profile_path: PathBuf,
    /// Whether logging is enabled
    pub enable_logging: bool,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            profile_path: PathBuf::from("crates/sandbox/profiles/minimal.sb"),
            enable_logging: true,
        }
    }
}

/// Sandbox event data model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxEvent {
    pub timestamp: DateTime<Utc>,
    pub sandbox_id: String,
    pub process: String,
    pub status: String,
    pub duration_sec: f64,
    pub writes: u32,
    pub spawns: u32,
    pub net_access: bool,
}

/// Backend trait for sandbox execution
trait Backend: Send + Sync {
    fn run(&self, cfg: &SandboxConfig, target: &str, args: &[&str]) -> Result<SandboxEvent>;
    fn poll(&self, _cfg: &SandboxConfig) -> Result<Vec<SandboxEvent>> {
        Ok(Vec::new())
    }
}

/// macOS backend (uses `sandbox-exec`)
struct MacBackend;
impl Backend for MacBackend {
    fn run(&self, cfg: &SandboxConfig, target: &str, args: &[&str]) -> Result<SandboxEvent> {
        let start = std::time::Instant::now();
        let output = Command::new("sandbox-exec")
            .arg("-f")
            .arg(&cfg.profile_path)
            .arg(target)
            .args(args)
            .output()?;

        let duration = start.elapsed().as_secs_f64();

        let status = if output.status.success() {
            "ok".to_string()
        } else {
            return Err(SandboxError::ExecFailure(
                String::from_utf8_lossy(&output.stderr).to_string(),
            )
            .into());
        };

        let event = SandboxEvent {
            timestamp: Utc::now(),
            sandbox_id: "mac_profile".into(),
            process: target.to_string(),
            status,
            duration_sec: duration,
            writes: 0,         // Phase II: parse logs for writes
            spawns: 0,         // Phase II: parse logs for spawns
            net_access: false, // Phase II: detect via hooks
        };

        if cfg.enable_logging {
            log_event(&event)?;
        }

        Ok(event)
    }
}

/// Linux backend (stub for Phase I)
struct LinuxBackend;
impl Backend for LinuxBackend {
    fn run(&self, _cfg: &SandboxConfig, target: &str, _args: &[&str]) -> Result<SandboxEvent> {
        Ok(SandboxEvent {
            timestamp: Utc::now(),
            sandbox_id: "linux_stub".into(),
            process: target.to_string(),
            status: "not_supported".into(),
            duration_sec: 0.0,
            writes: 0,
            spawns: 0,
            net_access: false,
        })
    }
}

/// Windows backend (stub for Phase I)
struct WindowsBackend;
impl Backend for WindowsBackend {
    fn run(&self, _cfg: &SandboxConfig, target: &str, _args: &[&str]) -> Result<SandboxEvent> {
        Ok(SandboxEvent {
            timestamp: Utc::now(),
            sandbox_id: "windows_stub".into(),
            process: target.to_string(),
            status: "not_supported".into(),
            duration_sec: 0.0,
            writes: 0,
            spawns: 0,
            net_access: false,
        })
    }
}

/// Null backend (catch-all)
struct NullBackend;
impl Backend for NullBackend {
    fn run(&self, _cfg: &SandboxConfig, _target: &str, _args: &[&str]) -> Result<SandboxEvent> {
        Err(SandboxError::Unsupported.into())
    }
}

/// Platform selector
fn backend() -> &'static dyn Backend {
    #[cfg(target_os = "macos")]
    {
        &MacBackend
    }
    #[cfg(target_os = "linux")]
    {
        &LinuxBackend
    }
    #[cfg(target_os = "windows")]
    {
        &WindowsBackend
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        &NullBackend
    }
}

/// Public API: run a process in sandbox
pub fn run_in_sandbox(
    cfg: &SandboxConfig,
    target: &str,
    args: &[&str],
) -> Result<SandboxEvent> {
    backend().run(cfg, target, args)
}

/// Public API: poll sandbox logs
pub fn poll_sandbox(cfg: &SandboxConfig) -> Result<Vec<SandboxEvent>> {
    backend().poll(cfg)
}

/// Append event to log file
fn log_event(event: &SandboxEvent) -> Result<()> {
    let _guard = LOG_LOCK.lock().unwrap();
    let serialized = serde_json::to_string(event)?;
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(LOG_PATH)?;
    writeln!(file, "{}", serialized)?;
    Ok(())
}