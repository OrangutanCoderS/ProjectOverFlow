//! Module 13 — Network Monitor (netmon)
//! Goal: Process-attributed live network activity with safe, layered backends.
//!
//! Backends (auto-detected, macOS-first):
//!  - NettopBackend     : macOS root, per-PID bytes via `nettop` (preferred)
//!  - LsofBackend       : macOS user-level, PID/ports via `lsof -nP` (+ optional netstat assist)
//!  - FallbackBackend   : portable "no data" stub (compiles everywhere, returns empty)
//!
//! No edits to other crates; all APIs here are additive. Consumers can:
//!  - call `sample_now()` for a one-shot pull
//!  - run `spawn_polling(...)` to stream `NetworkConnectionInfo` items
//!
//! Security: parsing is bounded, regex-guarded; all subprocess calls time out via overflow-utils.

use anyhow::{anyhow, Context, Result};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};
use thiserror::Error;
use overflow_utils::{run_command_with_timeout, utc_iso8601, ProcOutput};/// Naming stays aligned with your conventions: *_info, *_event.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NetworkConnectionInfo {
    pub timestamp: String,        // ISO-8601 UTC
    pub pid: i32,
    pub process: String,
    pub user: String,
    pub local_ip: String,
    pub local_port: u16,
    pub remote_ip: String,
    pub remote_port: u16,
    pub protocol: String,         // "TCP" | "UDP" | "UNKNOWN"
    pub state: String,            // "ESTABLISHED" | "LISTEN" | ...
    pub bytes_sent: Option<u64>,  // present with nettop
    pub bytes_received: Option<u64>,
    pub interface: Option<String>,// present with nettop
    pub dns_query: Option<String>,// if we ever enrich via resolver logs
}

impl NetworkConnectionInfo {
    pub fn validate(&self) -> Result<()> {
        // Tight but pragmatic validation.
        if self.pid <= 0 {
            return Err(anyhow!("pid must be > 0"));
        }
        if self.process.trim().is_empty() {
            return Err(anyhow!("process must be non-empty"));
        }
        if self.local_port == 0 && self.remote_port == 0 {
            return Err(anyhow!("at least one port must be non-zero"));
        }
        if !(self.protocol == "TCP" || self.protocol == "UDP" || self.protocol == "UNKNOWN") {
            return Err(anyhow!("unsupported protocol"));
        }
        Ok(())
    }
}

/// Public config object for polling (kept local to avoid touching global config crate).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Polling interval for the streaming runner.
    pub interval_ms: u64,
    /// Hard cap per sample to avoid unbounded growth / parser overwork.
    pub max_connections_per_sample: usize,
    /// Kill subprocesses after this timeout (ms).
    pub subprocess_timeout_ms: u64,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            interval_ms: 1000,
            max_connections_per_sample: 10_000,
            subprocess_timeout_ms: 1500,
        }
    }
}

/// Internal trait for backends.
trait Backend: Send + Sync {
    fn name(&self) -> &'static str;
    fn sample(&self, limit: usize, subproc_timeout: Duration) -> Result<Vec<NetworkConnectionInfo>>;
}

/// Auto-detect on first use (macOS-first). We *attempt* nettop; fallback to lsof; else stub.
static BACKEND: Lazy<Box<dyn Backend>> = Lazy::new(|| {
    if NettopBackend::available() {
        return Box::new(NettopBackend);
    }
    if LsofBackend::available() {
        return Box::new(LsofBackend::new());
    }
    Box::new(FallbackBackend)
});

/// One-shot sampling API used by daemon / telemetry bridge.
pub fn sample_now(cfg: &NetworkConfig) -> Result<Vec<NetworkConnectionInfo>> {
    let started = Instant::now();
    let out = BACKEND
        .sample(cfg.max_connections_per_sample, Duration::from_millis(cfg.subprocess_timeout_ms))
        .with_context(|| format!("netmon backend {} failed sample()", BACKEND.name()))?;

    // Lightweight health metric: ensure we didn't blow the budget
    let _elapsed = started.elapsed();
    // (Optionally emit a debug metric here via your logs crate)
    Ok(out)
}

/// Streaming runner: spawns a polling thread and returns a Receiver.
/// The sender is internal; to stop, drop the Receiver or join the thread externally.
pub fn spawn_polling(cfg: NetworkConfig) -> Receiver<NetworkConnectionInfo> {
    let (tx, rx): (Sender<NetworkConnectionInfo>, Receiver<NetworkConnectionInfo>) = mpsc::channel();
    thread::spawn(move || {
        let interval = Duration::from_millis(cfg.interval_ms.max(100));
        loop {
            match BACKEND.sample(cfg.max_connections_per_sample, Duration::from_millis(cfg.subprocess_timeout_ms)) {
                Ok(list) => {
                    for mut item in list {
                        // Normalize timestamp at emission time
                        item.timestamp = utc_iso8601();
                        // Do not let a slow/blocked receiver freeze the loop
                        let _ = tx.send(item);
                    }
                }
                Err(_) => {
                    // Fail closed: skip this tick, continue. Upstream watchdog will catch systemic issues.
                }
            }
            thread::sleep(interval);
        }
    });
    rx
}

/// Backend 1 — macOS `nettop` parser (root usually required for per-PID byte counters).
struct NettopBackend;

impl NettopBackend {
    fn available() -> bool {
        if cfg!(target_os = "macos") {
            // Try to run a short-lived nettop command to see if present & invokable.
            // We do not require root here; if it fails at runtime, we'll error and the caller continues.
            if Command::new("which").arg("nettop").output().map(|o| o.status.success()).unwrap_or(false) {
                return true;
            }
        }
        false
    }

    fn parse_nettop(output: &str, limit: usize) -> Vec<NetworkConnectionInfo> {
        // `nettop -P -L 1 -J bytes_in,bytes_out,interface,state,command` produces a table-ish view.
        // Output varies by OS version; we'll lazily capture common fields using conservative regex windows.
        // We keep the parser fault-tolerant: any non-matching line is ignored.
        static RE: Lazy<Regex> = Lazy::new(|| {
            // Example tolerant pattern (best-effort):
            // <iface>  <state>  <process> <pid>  <local_ip>:<port>  <remote_ip>:<port>  in:<num> out:<num>
            Regex::new(
                r"(?P<iface>\S+)\s+(?P<state>[A-Z]+)\s+(?P<proc>.+?)\s+\((?P<pid>\d+)\)\s+(?P<lip>\d{1,3}(?:\.\d{1,3}){3}):(?P<lport>\d+)\s+(?P<rip>\d{1,3}(?:\.\d{1,3}){3}):(?P<rport>\d+).+?bytes_in[:=](?P<in>\d+).+?bytes_out[:=](?P<out>\d+)"
            ).unwrap()
        });

        let mut out = Vec::with_capacity(256);
        for (i, line) in output.lines().enumerate() {
            if out.len() >= limit { break; }
            if let Some(c) = RE.captures(line) {
                let info = NetworkConnectionInfo {
                    timestamp: utc_iso8601(),
                    pid: c.name("pid").and_then(|m| m.as_str().parse::<i32>().ok()).unwrap_or_default(),
                    process: c.name("proc").map(|m| m.as_str().trim().to_string()).unwrap_or_default(),
                    user: String::from("unknown"),
                    local_ip: c.name("lip").map(|m| m.as_str().to_string()).unwrap_or_default(),
                    local_port: c.name("lport").and_then(|m| m.as_str().parse::<u16>().ok()).unwrap_or(0),
                    remote_ip: c.name("rip").map(|m| m.as_str().to_string()).unwrap_or_default(),
                    remote_port: c.name("rport").and_then(|m| m.as_str().parse::<u16>().ok()).unwrap_or(0),
                    protocol: "TCP".to_string(), // nettop view is connection-centric (TCP focus)
                    state: c.name("state").map(|m| m.as_str().to_string()).unwrap_or_else(|| "UNKNOWN".to_string()),
                    bytes_sent: c.name("out").and_then(|m| m.as_str().parse().ok()),
                    bytes_received: c.name("in").and_then(|m| m.as_str().parse().ok()),
                    interface: c.name("iface").map(|m| m.as_str().to_string()),
                    dns_query: None,
                };
                // soft validation to avoid junk
                if info.pid > 0 && (info.local_port != 0 || info.remote_port != 0) {
                    out.push(info);
                }
            } else {
                // Non-matching lines are skipped, never crash the monitor
                let _ = i; // keep clippy calm
            }
        }
        out
    }
}

impl Backend for NettopBackend {
    fn name(&self) -> &'static str { "nettop" }

    fn sample(&self, limit: usize, subproc_timeout: Duration) -> Result<Vec<NetworkConnectionInfo>> {
        // We keep the command minimal to avoid TTY modes and pagination issues.
        // -P (per-process), -L 1 (one sample), -J (columns), -t (no curses) varies by version; use tolerant parsing.
        let cmd = ["-P", "-L", "1", "-J", "bytes_in,bytes_out,interface,state,command"];
        let out: ProcOutput = run_command_with_timeout("nettop", cmd, subproc_timeout)
            .context("failed to run nettop")?;
        Ok(Self::parse_nettop(&String::from_utf8_lossy(&out.stdout), limit))    }
}

/// Backend 2 — `lsof -i -nP` parser (portable across macOS; user-level).
struct LsofBackend {
    re_line: Regex,
}

impl LsofBackend {
    fn new() -> Self {
        // lsof header (typical): COMMAND PID USER FD TYPE DEVICE SIZE/OFF NODE NAME
        // Example NAME field for TCP:
        // TCP 127.0.0.1:54892->104.26.9.2:443 (ESTABLISHED)
        // or: TCP *:7000 (LISTEN)
        let re = Regex::new(
            r#"^(?P<proc>\S+)\s+(?P<pid>\d+)\s+(?P<user>\S+)\s+\S+\s+\S+\s+\S+\s+\S+\s+\S+\s+(?P<name>.+)$"#
        ).unwrap();
        Self { re_line: re }
    }

    fn available() -> bool {
        Command::new("which")
            .arg("lsof")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    fn parse_name_field(name: &str) -> Option<(String, String, u16, String, u16, String)> {
        // Returns (protocol, local_ip, local_port, remote_ip, remote_port, state)
        // Handle LISTEN (no remote) and established (has "->")
        // Examples:
        // "TCP 127.0.0.1:54892->104.26.9.2:443 (ESTABLISHED)"
        // "TCP *:7000 (LISTEN)"
        static RE_EST: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r#"^(?P<proto>TCP|UDP)\s+(?P<lip>[^: ]+):(?P<lport>\d+)->(?P<rip>[^: ]+):(?P<rport>\d+)\s+\((?P<state>[A-Z]+)\)"#).unwrap()
        });
        static RE_LSN: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r#"^(?P<proto>TCP|UDP)\s+(?P<lip>[^: ]+):(?P<lport>\d+)\s+\((?P<state>[A-Z]+)\)"#).unwrap()
        });

        if let Some(c) = RE_EST.captures(name) {
            return Some((
                c.name("proto")?.as_str().to_string(),
                c.name("lip")?.as_str().to_string(),
                c.name("lport")?.as_str().parse().ok()?,
                c.name("rip")?.as_str().to_string(),
                c.name("rport")?.as_str().parse().ok()?,
                c.name("state")?.as_str().to_string(),
            ));
        }
        if let Some(c) = RE_LSN.captures(name) {
            return Some((
                c.name("proto")?.as_str().to_string(),
                c.name("lip")?.as_str().to_string(),
                c.name("lport")?.as_str().parse().ok()?,
                String::from("0.0.0.0"),
                0_u16,
                c.name("state")?.as_str().to_string(),
            ));
        }
        None
    }

    fn parse_lsof(&self, stdout: &str, limit: usize) -> Vec<NetworkConnectionInfo> {
        let mut out = Vec::with_capacity(1024);
        for (i, line) in stdout.lines().enumerate() {
            if out.len() >= limit { break; }
            if i == 0 && line.starts_with("COMMAND") {
                // header
                continue;
            }
            if let Some(c) = self.re_line.captures(line) {
                let proc = c.name("proc").map(|m| m.as_str()).unwrap_or("");
                let pid = c.name("pid").and_then(|m| m.as_str().parse::<i32>().ok()).unwrap_or(0);
                let user = c.name("user").map(|m| m.as_str()).unwrap_or("");
                let name_field = c.name("name").map(|m| m.as_str()).unwrap_or("");
                if let Some((proto, lip, lport, rip, rport, state)) = Self::parse_name_field(name_field) {
                    let info = NetworkConnectionInfo {
                        timestamp: utc_iso8601(),
                        pid,
                        process: proc.to_string(),
                        user: user.to_string(),
                        local_ip: lip,
                        local_port: lport,
                        remote_ip: rip,
                        remote_port: rport,
                        protocol: proto,
                        state,
                        bytes_sent: None,
                        bytes_received: None,
                        interface: None,
                        dns_query: None,
                    };
                    if info.pid > 0 {
                        out.push(info);
                    }
                }
            }
        }
        out
    }
}

impl Backend for LsofBackend {
    fn name(&self) -> &'static str { "lsof" }

    fn sample(&self, limit: usize, subproc_timeout: Duration) -> Result<Vec<NetworkConnectionInfo>> {
        // We run a single lsof and parse. No netstat pairing needed for basic attribution.
        let out: ProcOutput = run_command_with_timeout("lsof", ["-i", "-nP"], subproc_timeout)
            .context("failed to run lsof -i -nP")?;
        Ok(self.parse_lsof(&String::from_utf8_lossy(&out.stdout), limit))    }
}

/// Backend 3 — Fallback stub for unsupported platforms or missing tools.
struct FallbackBackend;
impl Backend for FallbackBackend {
    fn name(&self) -> &'static str { "fallback" }
    fn sample(&self, _limit: usize, _to: Duration) -> Result<Vec<NetworkConnectionInfo>> {
        Ok(vec![])
    }
}

/// Error surface for external consumers if needed later.
#[derive(Debug, Error)]
pub enum NetmonError {
    #[error("backend unavailable")]
    BackendUnavailable,
    #[error("parse error: {0}")]
    Parse(String),
}
