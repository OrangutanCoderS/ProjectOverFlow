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
            if Command::new("which").arg("nettop").output().map(|o| o.status.success()).unwrap_or(false) {
                return true;
            }
        }
        false
    }

    fn parse_csv(output: &str, limit: usize) -> Vec<NetworkConnectionInfo> {
        let mut out = Vec::with_capacity(256);

        for (i, line) in output.lines().enumerate() {
            if i == 0 { continue; } // skip header
            if out.len() >= limit { break; }
            let cols: Vec<&str> = line.split(',').collect();
            if cols.len() < 6 { continue; }

            // columns: time,conn,iface,state,bytes_in,bytes_out,...
            let conn_field = cols[1];
            let iface      = cols[2];
            let state      = cols[3];
            let bytes_in   = cols[4].parse::<u64>().ok();
            let bytes_out  = cols.get(5).and_then(|v| v.parse::<u64>().ok());

            // Protocol
            let mut protocol = "UNKNOWN".to_string();
            if conn_field.starts_with("tcp") { protocol = "TCP".to_string(); }
            if conn_field.starts_with("udp") { protocol = "UDP".to_string(); }

            // Parse IP/ports
            let mut local_ip = String::new();
            let mut local_port = 0u16;
            let mut remote_ip = String::new();
            let mut remote_port = 0u16;

            if let Some((l, r)) = conn_field.split_once("<->") {
                if let Some((ip, port)) = l.rsplit_once(':') {
                    local_ip = ip.to_string();
                    local_port = port.parse().unwrap_or(0);
                }
                if let Some((ip, port)) = r.rsplit_once(':') {
                    remote_ip = ip.to_string();
                    remote_port = port.parse().unwrap_or(0);
                }
            }

            out.push(NetworkConnectionInfo {
                timestamp: utc_iso8601(),
                pid: 0, // to be enriched later
                process: conn_field.to_string(),
                user: "unknown".to_string(),
                local_ip,
                local_port,
                remote_ip,
                remote_port,
                protocol,
                state: state.to_string(),
                bytes_sent: bytes_out,
                bytes_received: bytes_in,
                interface: Some(iface.to_string()),
                dns_query: None,
            });
        }
        out
    }

    fn enrich_with_lsof(conns: Vec<NetworkConnectionInfo>, subproc_timeout: Duration)
        -> Vec<NetworkConnectionInfo>
    {
        use std::collections::HashMap;
        let mut map: HashMap<(String, u16, String, u16, String), (i32, String, String, String)> = HashMap::new();

        if let Ok(out) = run_command_with_timeout("lsof", ["-i", "-nP"], subproc_timeout) {
            let text = String::from_utf8_lossy(&out.stdout);
            for line in text.lines().skip(1) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() < 9 { continue; }

                let pid = parts[1].parse::<i32>().unwrap_or(0);
                let user = parts[2].to_string();
                let proc_name = parts[0].to_string();
                let name_field = parts[8..].join(" ");

                if let Some((proto, lip, lport, rip, rport, state)) =
                    LsofBackend::parse_name_field(&name_field)
                {
                    map.insert(
                        (lip.clone(), lport, rip.clone(), rport, proto.clone()),
                        (pid, proc_name, user, state),
                    );
                }
            }
        }

        conns.into_iter().map(|mut c| {
            if let Some((pid, proc, user, state)) =
                map.get(&(c.local_ip.clone(), c.local_port, c.remote_ip.clone(), c.remote_port, c.protocol.clone()))
            {
                c.pid = *pid;
                c.process = proc.clone();
                c.user = user.clone();
                c.state = state.clone();
            }
            c
        }).collect()
    }
}

impl Backend for NettopBackend {
    fn name(&self) -> &'static str { "nettop" }

    fn sample(&self, limit: usize, subproc_timeout: Duration) -> Result<Vec<NetworkConnectionInfo>> {
        // CSV mode (macOS 26+), force raw IPs with -n
        let csv_out: ProcOutput = run_command_with_timeout(
            "nettop",
            ["-n", "-L", "1"],   // 👈 added -n here
            subproc_timeout,
        ).context("failed to run nettop -n -L 1")?;

        let parsed_csv = NettopBackend::parse_csv(&String::from_utf8_lossy(&csv_out.stdout), limit);

        // Enrich with lsof (for PID, process name, state, user)
        let enriched = NettopBackend::enrich_with_lsof(parsed_csv, subproc_timeout);

        Ok(enriched)
    }
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
pub fn backend_name() -> &'static str {
    BACKEND.name()
}
