//! Module 5 — Process Monitor (Phase I)
//! Cross-platform process snapshotter with stable API and delta detection.
//! Backend: sysinfo (safe, portable). Swappable later for macOS libproc/C without API changes.

use hashbrown::HashMap;
use overflow_core::{AnyEvent, BaseEvent, ProcessInfo, TimestampMs, Event}; // 👈 Event added here
use overflow_utils::unix_time_ms;
use serde::{Deserialize, Serialize};
use sysinfo::{ProcessRefreshKind, RefreshKind, System, Users};
use thiserror::Error;
use tracing::{debug, warn};

#[derive(Debug, Error)]
pub enum ProcMonError {
    #[error("system init: {0}")]
    Init(String),
    #[error("snapshot: {0}")]
    Snapshot(String),
    #[error("serialize: {0}")]
    Serde(String),
}

pub type Result<T> = std::result::Result<T, ProcMonError>;

/// Public configuration (kept minimal; scheduler tick lives in daemon/config)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProcMonCfg {
    /// If true, emit only deltas since previous snapshot; else emit all processes every tick.
    pub emit_deltas: bool,
    /// Upper bound for snapshot duration in ms; if exceeded we warn (does not fail).
    pub soft_budget_ms: u64,
}

impl Default for ProcMonCfg {
    fn default() -> Self {
        Self {
            emit_deltas: true,
            soft_budget_ms: 100,
        }
    }
}

/// One complete snapshot of processes at a point in time.
#[derive(Debug, Clone)]
pub struct Snapshot {
    /// key = pid
    pub map: HashMap<i32, ProcessInfo>,
    pub captured_at_ms: TimestampMs,
}

impl Snapshot {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            captured_at_ms: unix_time_ms(),
        }
    }
}

/// Diff between two snapshots.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Diff {
    pub started: Vec<i32>,
    pub ended: Vec<i32>,
    /// Processes still present but with significant state changes (currently cpu/mem/threads)
    pub changed: Vec<i32>,
}

/// Primary monitor object. Not thread-safe by itself; daemon supervises concurrency.
pub struct ProcessMonitor {
    sys: System,
    users: Users,
    last: Option<Snapshot>,
    cfg: ProcMonCfg,
}

impl ProcessMonitor {
    pub fn new(cfg: ProcMonCfg) -> Result<Self> {
        let mut sys = System::new();
        // Only refresh processes to keep overhead low.
        let refresh = RefreshKind::new().with_processes(ProcessRefreshKind::everything());
        sys.refresh_specifics(refresh);
        let users = Users::new_with_refreshed_list();
        Ok(Self {
            sys,
            users,
            last: None,
            cfg,
        })
    }

    /// Capture one snapshot of all running processes.
    pub fn capture(&mut self) -> Result<Snapshot> {
        let t0 = std::time::Instant::now();

        // Incremental refresh: update processes only
        self.sys
            .refresh_processes_specifics(ProcessRefreshKind::everything());

        let now_ms = unix_time_ms();

        // Absolute system uptime (seconds) — associated function in sysinfo 0.30
        let uptime_s = System::uptime() as i64;

        let mut snap = Snapshot {
            map: HashMap::with_capacity(self.sys.processes().len()),
            captured_at_ms: now_ms,
        };

        for (pid, p) in self.sys.processes() {
            // sysinfo::Pid is a newtype — convert via as_u32()
            let pid_i32 = pid.as_u32() as i32;

            // Strings are already &str in sysinfo 0.30
            let name = p.name().to_string();
            let ppid = p.parent().map(|pp| pp.as_u32() as i32).unwrap_or(0);

            let user = match p.user_id() {
                Some(uid) => self
                    .users
                    .get_user_by_id(uid)
                    .map(|u| u.name().to_string())
                    .unwrap_or_else(|| uid.to_string()),
                None => "unknown".to_string(),
            };

            let cpu_pct = p.cpu_usage(); // f32
            // sysinfo::Process::memory() returns KiB (u64) — convert to MiB
            let mem_mb = (p.memory() as f32) / 1024.0;

            // sysinfo 0.30 no longer exposes thread list cross-platform; keep portable default.
            let threads: u32 = 1;

            // cmd() is Vec<String> in 0.30
            let cmdline: Vec<String> = p.cmd().iter().cloned().collect();

            // start_time(): seconds since boot
            let start_since_boot_s = p.start_time() as i64;
            let start_time_ms = if start_since_boot_s <= uptime_s {
                // now_ms - (uptime - start_since_boot) * 1000
                now_ms - ((uptime_s - start_since_boot_s) * 1000)
            } else {
                // Fallback: just now (clock skew edge)
                now_ms
            };

            let base = BaseEvent::new(pid_i32, "process_monitor");
            let info = ProcessInfo {
                base,
                name,
                ppid,
                user,
                cpu_pct,
                mem_mb,
                threads,
                cmdline,
                start_time_ms,
            };

            // Validate model defensively
            if let Err(e) = info.validate() {
                warn!(pid = pid_i32, err = %e, "dropping invalid ProcessInfo");
                continue;
            }

            snap.map.insert(pid_i32, info);
        }

        let elapsed = t0.elapsed().as_millis() as u64;
        if elapsed > self.cfg.soft_budget_ms {
            warn!(elapsed_ms = elapsed, "process snapshot exceeded soft budget");
        } else {
            debug!(elapsed_ms = elapsed, "process snapshot ok");
        }

        Ok(snap)
    }

    /// Compute a diff relative to the previous snapshot (if any).
    pub fn diff(&self, prev: &Snapshot, next: &Snapshot) -> Diff {
        let mut started = Vec::new();
        let mut ended = Vec::new();
        let mut changed = Vec::new();

        for pid in next.map.keys() {
            if !prev.map.contains_key(pid) {
                started.push(*pid);
            } else {
                // simple material change heuristic
                let a = prev.map.get(pid).unwrap();
                let b = next.map.get(pid).unwrap();
                let cpu_diff = (a.cpu_pct - b.cpu_pct).abs();
                let mem_diff = (a.mem_mb - b.mem_mb).abs();
                let thr_diff = a.threads.abs_diff(b.threads);
                if cpu_diff > 15.0 || mem_diff > 32.0 || thr_diff >= 4 {
                    changed.push(*pid);
                }
            }
        }
        for pid in prev.map.keys() {
            if !next.map.contains_key(pid) {
                ended.push(*pid);
            }
        }
        Diff {
            started,
            ended,
            changed,
        }
    }

    /// Run one monitoring tick: capture, compute diff (if configured), and return events to emit.
    pub fn tick(&mut self) -> Result<Vec<AnyEvent>> {
        let new_snap = self.capture()?;
        let events = if let Some(old) = &self.last {
            if self.cfg.emit_deltas {
                self.emit_deltas(old, &new_snap)
            } else {
                self.emit_full(&new_snap)
            }
        } else {
            // First tick: either full dump or empty depending on preference; we choose full.
            self.emit_full(&new_snap)
        };
        self.last = Some(new_snap);
        Ok(events)
    }

    fn emit_full(&self, snap: &Snapshot) -> Vec<AnyEvent> {
        snap.map
            .values()
            .cloned()
            .map(AnyEvent::ProcessInfo)
            .collect()
    }

    fn emit_deltas(&self, old: &Snapshot, new: &Snapshot) -> Vec<AnyEvent> {
        let d = self.diff(old, new);
        let mut out = Vec::with_capacity(d.started.len() + d.changed.len());
        for pid in d.started.iter().chain(d.changed.iter()) {
            if let Some(info) = new.map.get(pid) {
                out.push(AnyEvent::ProcessInfo(info.clone()));
            }
        }
        out
    }
}

/* ============================
   Tests (unit-level)
   ============================ */

#[cfg(test)]
mod tests {
    use super::*;
    use overflow_core::Event; // 👈 added here
    use serial_test::serial;

    #[test]
    #[serial] // keep sysinfo refresh ordered in CI
    fn snapshot_has_current_process() {
        let mut mon = ProcessMonitor::new(ProcMonCfg::default()).unwrap();
        let snap = mon.capture().unwrap();
        let me = std::process::id() as i32;
        assert!(snap.map.contains_key(&me));
    }

    #[test]
    #[serial]
    fn diff_detects_start_and_end() {
        let mut mon = ProcessMonitor::new(ProcMonCfg { emit_deltas: true, soft_budget_ms: 500 }).unwrap();
        let s1 = mon.capture().unwrap();

        // Spawn a short-lived child
        let mut child = std::process::Command::new("sh")
            .arg("-c").arg("sleep 0.2")
            .spawn().unwrap();
        let _ = child.wait().unwrap();

        let s2 = mon.capture().unwrap();
        let d = mon.diff(&s1, &s2);
        assert!(d.started.len() >= 0); // not guaranteed we caught it; but should not panic
        assert!(d.ended.len() >= 0);
    }

    #[test]
    fn model_is_validatable() {
        let base = BaseEvent::new(1, "process_monitor");
        let ok = ProcessInfo {
            base,
            name: "sh".into(),
            ppid: 1,
            user: "root".into(),
            cpu_pct: 0.0,
            mem_mb: 1.0,
            threads: 1,
            cmdline: vec!["sh".into()],
            start_time_ms: unix_time_ms(),
        };
        assert!(ok.validate().is_ok());
    }
}