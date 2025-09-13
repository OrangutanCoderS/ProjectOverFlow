/*!
 Module 15 — Sensor Log Reader (sensorlog)
 Goal: Poll macOS TCC database for privacy-sensitive sensor access history (camera, mic, screen capture),
 and emit structured records. Linux/Windows provide a safe fallback (empty vector).
 No edits to other crates; strictly additive.

 Design choices:
 - macOS backend uses `sqlite3` CLI (guarded by hardened timeout) to avoid linking libsqlite in the workspace.
 - We probe TCC schema and try multiple column variants to tolerate OS version drift.
 - Parsing is CSV-based with Regex validation and per-sample caps to avoid unbounded memory.
*/

use anyhow::{anyhow, Context, Result};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};
use thiserror::Error;

// overflow-utils crate: lib name is overflow_utils, package name overflow-utils
use overflow_utils::{run_command_with_timeout, utc_iso8601, ProcOutput};

/// Public model — consistent with *_info naming.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SensorAccessInfo {
    pub timestamp: String,        // ISO-8601 UTC (time we observed/logged the row)
    pub sensor: String,           // "camera" | "microphone" | "screen_capture" | "unknown"
    pub access_type: String,      // "read" | "control" | "unknown"
    pub pid: Option<i32>,         // TCC doesn't store pid; Phase I -> None
    pub process: Option<String>,  // best-effort from client (bundle id)
    pub last_used_unix: Option<i64>, // seconds since epoch if available
    pub foreground: Option<bool>, // Not provided by TCC; Phase I -> None
}

impl SensorAccessInfo {
    pub fn validate(&self) -> Result<()> {
        if self.timestamp.is_empty() {
            return Err(anyhow!("timestamp must be non-empty"));
        }
        if self.sensor.is_empty() {
            return Err(anyhow!("sensor must be non-empty"));
        }
        if !(self.access_type == "read" || self.access_type == "control" || self.access_type == "unknown") {
            return Err(anyhow!("invalid access_type"));
        }
        Ok(())
    }
}

/// Local config — does not alter global configs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorConfig {
    pub poll_interval_ms: u64,       // default: 2000ms
    pub subprocess_timeout_ms: u64,  // `sqlite3` timeout guard
    pub max_entries_per_sample: usize, // cap rows per poll
    /// Optional override to TCC db path (for tests). If None, auto-detect platform defaults.
    pub tcc_db_path: Option<PathBuf>,
}

impl Default for SensorConfig {
    fn default() -> Self {
        Self {
            poll_interval_ms: 2000,
            subprocess_timeout_ms: 2000,
            max_entries_per_sample: 2048,
            tcc_db_path: None,
        }
    }
}

/// Public API: one-shot polling. Returns 0..N rows (bounded by cap).
pub fn poll_sensors(cfg: &SensorConfig) -> Result<Vec<SensorAccessInfo>> {
    BACKEND.sample(cfg)
}

/// Public API: spawn a background polling loop, streaming individual events.
pub fn spawn_sensor_reader(cfg: SensorConfig) -> Receiver<SensorAccessInfo> {
    let (tx, rx): (Sender<SensorAccessInfo>, Receiver<SensorAccessInfo>) = mpsc::channel();
    thread::spawn(move || {
        let interval = Duration::from_millis(cfg.poll_interval_ms.max(100));
        loop {
            match BACKEND.sample(&cfg) {
                Ok(list) => {
                    for mut ev in list {
                        // normalize timestamp to "now" (observation time)
                        ev.timestamp = utc_iso8601();
                        let _ = tx.send(ev);
                    }
                }
                Err(_) => { /* fail closed */ }
            }
            thread::sleep(interval);
        }
    });
    rx
}

/* ============================
   Backend selection
   ============================ */

trait Backend: Send + Sync {
    fn name(&self) -> &'static str;
    fn sample(&self, cfg: &SensorConfig) -> Result<Vec<SensorAccessInfo>>;
}

static BACKEND: Lazy<Box<dyn Backend>> = Lazy::new(|| {
    if MacTccBackend::available() {
        return Box::new(MacTccBackend);
    }
    if LinuxFallback::available() {
        return Box::new(LinuxFallback);
    }
    if WindowsFallback::available() {
        return Box::new(WindowsFallback);
    }
    Box::new(NullBackend)
});

/* ============================
   macOS TCC backend
   ============================ */

struct MacTccBackend;

impl MacTccBackend {
    fn available() -> bool {
        if cfg!(target_os = "macos") {
            // We require sqlite3 CLI to be present.
            return Command::new("which").arg("sqlite3").output().map(|o| o.status.success()).unwrap_or(false);
        }
        false
    }

    fn default_tcc_paths() -> Vec<PathBuf> {
        // Order: system-wide, then per-user
        let mut v = Vec::new();
        // system db
        v.push(PathBuf::from("/Library/Application Support/com.apple.TCC/TCC.db"));
        // user db
        if let Some(home) = std::env::var_os("HOME") {
            v.push(PathBuf::from(home).join("Library/Application Support/com.apple.TCC/TCC.db"));
        }
        v
    }

    /// Try a series of queries to tolerate schema drift across macOS versions.
    /// We prefer CSV mode (comma separator). Columns: client, service, last_used (or last_modified/last_seen).
    fn try_sqlite_queries(db: &PathBuf, timeout: Duration, limit: usize) -> Result<String> {
    // Candidate SQL strings (ordered by preference).
    let sqls = [
        "SELECT client,service,last_used FROM access ORDER BY last_used DESC LIMIT ?1;",
        "SELECT client,service,last_modified FROM access ORDER BY last_modified DESC LIMIT ?1;",
        "SELECT client,service,last_seen FROM access ORDER BY last_seen DESC LIMIT ?1;",
        "SELECT client,service,created_at FROM access ORDER BY created_at DESC LIMIT ?1;",
    ];

    let lim = limit.min(10_000); // safety
    let lim_s = lim.to_string();

    for sql in &sqls {
        // Own the conversions so they live long enough
        let db_str = db.to_string_lossy().to_string();
        let sql_str = sql.replace("?1", &lim_s);

        // Build the args slice using &str references to owned Strings
        let args: [&str; 3] = ["-csv", db_str.as_str(), sql_str.as_str()];

        let res = run_command_with_timeout("sqlite3", args, timeout);
        match res {
            Ok(out) if out.status_code == 0 && !out.stdout.is_empty() => {
                return Ok(String::from_utf8_lossy(&out.stdout).to_string());
            }
            _ => continue, // try next query form
        }
    }

    Err(anyhow!("no compatible TCC access columns were readable"))
}

    fn parse_csv(csv: &str, cfg: &SensorConfig) -> Vec<SensorAccessInfo> {
        // Tolerant CSV: 3 columns per line: client,service,last_used
        // Example line: "us.zoom.xos","kTCCServiceMicrophone","1726106000"
        static RE_LINE: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r#"^\s*"?(?P<client>[^",]*)"?\s*,\s*"?(?P<service>[^",]*)"?\s*,\s*"?(?P<ts>-?\d+)"?\s*$"#).unwrap()
        });

        let mut out = Vec::with_capacity(256);
        for (i, line) in csv.lines().enumerate() {
            if out.len() >= cfg.max_entries_per_sample { break; }
            if line.trim().is_empty() { continue; }
            if let Some(c) = RE_LINE.captures(line) {
                let client = c.name("client").map(|m| m.as_str().to_string()).unwrap_or_default();
                let service = c.name("service").map(|m| m.as_str().to_string()).unwrap_or_default();
                let ts = c.name("ts").and_then(|m| m.as_str().parse::<i64>().ok());
                let sensor = map_service_to_sensor(&service);
                let access_type = map_service_access_type(&service);

                let ev = SensorAccessInfo {
                    timestamp: utc_iso8601(), // observation time (db may have last_used)
                    sensor,
                    access_type,
                    pid: None,
                    process: if client.is_empty() { None } else { Some(client) },
                    last_used_unix: ts,
                    foreground: None,
                };
                // Minimal sanity validation
                if ev.sensor != "unknown" {
                    out.push(ev);
                }
            } else {
                let _ = i; // skip non-matching line safely
            }
        }
        out
    }
}

impl Backend for MacTccBackend {
    fn name(&self) -> &'static str { "macos_tcc_sqlite" }

    fn sample(&self, cfg: &SensorConfig) -> Result<Vec<SensorAccessInfo>> {
        let started = Instant::now();
        // Decide which DB path to hit
        let candidates = if let Some(p) = &cfg.tcc_db_path {
            vec![p.clone()]
        } else {
            Self::default_tcc_paths()
        };

        for db in candidates {
            if !db.exists() { continue; }
            // Ensure sqlite3 is present (already checked in available()), and query.
            let csv = Self::try_sqlite_queries(&db, Duration::from_millis(cfg.subprocess_timeout_ms), cfg.max_entries_per_sample)
                .with_context(|| format!("sqlite3 TCC query failed for {}", db.display()))?;
            let mut rows = Self::parse_csv(&csv, cfg);
            // Health gate: ensure we didn't exceed cap and we parsed something meaningful
            if !rows.is_empty() {
                let _elapsed = started.elapsed();
                return Ok(rows);
            }
        }

        // If no DBs yielded data, return empty but not an error (fail-closed posture).
        Ok(vec![])
    }
}

/* ============================
   Linux and Windows fallbacks
   ============================ */

struct LinuxFallback;
impl LinuxFallback {
    fn available() -> bool { cfg!(target_os = "linux") }
}
impl Backend for LinuxFallback {
    fn name(&self) -> &'static str { "linux_fallback" }
    fn sample(&self, _cfg: &SensorConfig) -> Result<Vec<SensorAccessInfo>> {
        Ok(vec![]) // Phase I: not supported; future: audit/auditd/journald taps
    }
}

struct WindowsFallback;
impl WindowsFallback {
    fn available() -> bool { cfg!(target_os = "windows") }
}
impl Backend for WindowsFallback {
    fn name(&self) -> &'static str { "windows_fallback" }
    fn sample(&self, _cfg: &SensorConfig) -> Result<Vec<SensorAccessInfo>> {
        Ok(vec![]) // Phase I: not supported; future: ETW/WMI providers
    }
}

/// Null backend — cross-platform build safety.
struct NullBackend;
impl Backend for NullBackend {
    fn name(&self) -> &'static str { "null" }
    fn sample(&self, _cfg: &SensorConfig) -> Result<Vec<SensorAccessInfo>> { Ok(vec![]) }
}

/* ============================
   Mapping helpers
   ============================ */

fn map_service_to_sensor(service: &str) -> String {
    match service {
        "kTCCServiceCamera" => "camera".to_string(),
        "kTCCServiceMicrophone" => "microphone".to_string(),
        "kTCCServiceScreenCapture" => "screen_capture".to_string(),
        // common alternates (Apple occasionally changes names)
        "kTCCServiceListenEvent" => "microphone".to_string(),
        "kTCCServiceMediaLibrary" => "unknown".to_string(),
        _ => "unknown".to_string(),
    }
}

fn map_service_access_type(service: &str) -> String {
    match service {
        "kTCCServiceCamera" | "kTCCServiceMicrophone" | "kTCCServiceScreenCapture" => "read".to_string(),
        _ => "unknown".to_string(),
    }
}

/* ============================
   Public error surface
   ============================ */
#[derive(Debug, Error)]
pub enum SensorlogError {
    #[error("backend unavailable")]
    BackendUnavailable,
    #[error("parse error: {0}")]
    Parse(String),
}
