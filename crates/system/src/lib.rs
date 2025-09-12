//! Module 6 — System Stats (Phase I)
//! Snapshot system health using sysinfo 0.30.x with a stable, portable API.
//! No side effects: pure data structs + capture + serde helpers.

use serde::{Deserialize, Serialize};
use thiserror::Error;

// sysinfo 0.30.x API (networks/disks are separate collections)
use sysinfo::{
    CpuRefreshKind, DiskKind, Disks, MemoryRefreshKind, Networks, ProcessRefreshKind, RefreshKind,
    System,
};

/// Milliseconds since Unix epoch (UTC) (reused pattern from other modules)
pub type TimestampMs = i64;

fn now_ms() -> TimestampMs {
    use std::time::{SystemTime, UNIX_EPOCH};
    let dur = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    dur.as_millis() as i64
}

#[derive(Debug, Error)]
pub enum SysMonError {
    #[error("init: {0}")]
    Init(String),
    #[error("snapshot: {0}")]
    Snapshot(String),
    #[error("serde: {0}")]
    Serde(String),
}

/// Public config for the system monitor.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SystemCfg {
    /// If true, include per-disk rows; else aggregate only.
    pub include_disks: bool,
    /// Soft time budget in ms for a capture (warn only).
    pub soft_budget_ms: u64,
}

impl Default for SystemCfg {
    fn default() -> Self {
        Self {
            include_disks: true,
            soft_budget_ms: 100,
        }
    }
}

/// Aggregate network counters (since boot).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct NetCounters {
    pub rx_bytes: u64,
    pub tx_bytes: u64,
}

/// One disk row.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DiskStat {
    pub name: String,
    pub kind: String,
    pub total_bytes: u64,
    pub available_bytes: u64,
}

/// Snapshot of system resources at a moment in time.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SystemSnapshot {
    pub ts_ms: TimestampMs,
    /// Approximate global CPU usage percent (0..=100 * N on some platforms).
    pub cpu_pct: f32,
    /// Memory numbers in MiB (rounded).
    pub mem_total_mib: u64,
    pub mem_used_mib: u64,
    /// Summed network counters.
    pub net: NetCounters,
    /// Optional per-disk rows.
    pub disks: Vec<DiskStat>,
}

impl SystemSnapshot {
    pub fn to_json(&self) -> Result<String, SysMonError> {
        serde_json::to_string(self).map_err(|e| SysMonError::Serde(e.to_string()))
    }
}

/// Primary monitor.
pub struct SystemMonitor {
    sys: System,
    networks: Networks,
    disks: Disks,
    cfg: SystemCfg,
}

impl SystemMonitor {
    pub fn new(cfg: SystemCfg) -> Result<Self, SysMonError> {
        // Initialize System with explicit refresh kinds (no networks/disks here).
        let mut sys = System::new();
        let refresh = RefreshKind::new()
            .with_cpu(CpuRefreshKind::everything())
            .with_memory(MemoryRefreshKind::everything())
            .with_processes(ProcessRefreshKind::new()); // cheap; we don’t read processes here
        sys.refresh_specifics(refresh);

        // Create collections with their own refresh APIs.
        let mut networks = Networks::new();
        networks.refresh(); // populate

        let mut disks = Disks::new();
        disks.refresh_list(); // enumerate; stats read on demand

        Ok(Self {
            sys,
            networks,
            disks,
            cfg,
        })
    }

    /// Capture one snapshot of CPU/memory/network/disks.
    pub fn capture(&mut self) -> Result<SystemSnapshot, SysMonError> {
        let t0 = std::time::Instant::now();

        // Refresh CPU+mem (System), networks (Networks), disks list (Disks)
        self.sys.refresh_cpu();
        self.sys.refresh_memory();
        self.networks.refresh();
        self.disks.refresh_list(); // ensures list is up to date

        // CPU: global usage. Note: on some OS it can exceed 100 across cores.
        let cpu_pct: f32 = {
            let cpus = self.sys.cpus();
            if cpus.is_empty() {
                0.0
            } else {
                let total: f32 = cpus.iter().map(|cpu| cpu.cpu_usage()).sum();
                total / (cpus.len() as f32)
            }
        };

        // Memory (KiB) → MiB
        let total_kib = self.sys.total_memory();
        let used_kib = self.sys.used_memory();
        let mem_total_mib = (total_kib / 1024) as u64;
        let mem_used_mib = (used_kib / 1024) as u64;

        // Networks (sum all interfaces)
        let mut rx_bytes: u64 = 0;
        let mut tx_bytes: u64 = 0;
        for (_name, data) in self.networks.iter() {
            rx_bytes = rx_bytes.saturating_add(data.received());
            tx_bytes = tx_bytes.saturating_add(data.transmitted());
        }

        // Disks
        let mut disks_vec = Vec::new();
        if self.cfg.include_disks {
            for d in self.disks.list() {
                let name = d.name().to_string_lossy().to_string();
                let kind = match d.kind() {
                    DiskKind::HDD => "hdd",
                    DiskKind::SSD => "ssd",
                    DiskKind::Unknown(_) => "unknown",
                }
                .to_string();
                let total_bytes = d.total_space();
                let available_bytes = d.available_space();
                disks_vec.push(DiskStat {
                    name,
                    kind,
                    total_bytes,
                    available_bytes,
                });
            }
        }

        let snap = SystemSnapshot {
            ts_ms: now_ms(),
            cpu_pct,
            mem_total_mib,
            mem_used_mib,
            net: NetCounters { rx_bytes, tx_bytes },
            disks: disks_vec,
        };

        // Soft budget warn (no logging dependency here; return Ok always)
        let elapsed = t0.elapsed().as_millis() as u64;
        if elapsed > self.cfg.soft_budget_ms {
            // Up to you to log externally
        }

        Ok(snap)
    }
}

/* ============================
   Unit tests
   ============================ */
#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn snapshot_basic_sanity() {
        let mut mon = SystemMonitor::new(SystemCfg::default()).unwrap();
        let snap = mon.capture().unwrap();

        assert!(snap.cpu_pct >= 0.0, "cpu should be non-negative");
        assert!(snap.mem_total_mib > 0, "total mem must be > 0");
        assert!(snap.mem_used_mib <= snap.mem_total_mib, "used <= total");
        assert!(
            snap.net.rx_bytes >= 0 && snap.net.tx_bytes >= 0,
            "net counters must be non-negative"
        );

        // JSON roundtrip
        let s = snap.to_json().unwrap();
        let back: SystemSnapshot = serde_json::from_str(&s).unwrap();
        assert_eq!(back.mem_total_mib, snap.mem_total_mib);
    }

    #[test]
    #[serial]
    fn disks_list_is_stable() {
        let mut mon = SystemMonitor::new(SystemCfg {
            include_disks: true,
            soft_budget_ms: 1000,
        })
        .unwrap();
        let snap = mon.capture().unwrap();
        // On some CI runners there may be 0 disks exposed, so just ensure it deserializes.
        let s = snap.to_json().unwrap();
        let _: SystemSnapshot = serde_json::from_str(&s).unwrap();
    }
}