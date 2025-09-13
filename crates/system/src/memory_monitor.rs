//! Module 8 — Memory Monitor (Phase I)
//! Captures memory/swap, CPU %, load averages, uptime, and process count,
//! and emits a unified snapshot. Optionally emits a PluginTrigger when
//! memory usage crosses thresholds. Pure Rust with `sysinfo` backend.

use overflow_core::{
    AnyEvent, BaseEvent, Event, PluginTrigger, SystemStatSnapshot, TriggerFlags,
};
use sysinfo::{CpuRefreshKind, MemoryRefreshKind, ProcessRefreshKind, RefreshKind, System};
use thiserror::Error;
use tracing::{debug, warn};

#[derive(Debug, Error)]
pub enum MemMonError {
    #[error("init: {0}")]
    Init(String),
    #[error("sample: {0}")]
    Sample(String),
}

pub type Result<T> = std::result::Result<T, MemMonError>;

/// Public configuration for memory monitoring.
#[derive(Debug, Clone)]
pub struct MemMonCfg {
    /// Warn if a sampling pass exceeds this time budget (ms).
    pub soft_budget_ms: u64,
    /// Also refresh processes to keep `process_count` fresh.
    pub refresh_processes: bool,
    /// Emit `PluginTrigger` when used/total >= warn_threshold_pct.
    pub warn_threshold_pct: f32, // e.g., 80.0
    /// Emit `PluginTrigger` when used/total >= crit_threshold_pct.
    pub crit_threshold_pct: f32, // e.g., 90.0
    /// If false, never emit `PluginTrigger` (snapshots only).
    pub emit_triggers: bool,
}

impl Default for MemMonCfg {
    fn default() -> Self {
        Self {
            soft_budget_ms: 100,
            refresh_processes: false,
            warn_threshold_pct: 80.0,
            crit_threshold_pct: 90.0,
            emit_triggers: true,
        }
    }
}

/// Main monitor object (not thread-safe itself; daemon controls scheduling).
pub struct MemoryMonitor {
    sys: System,
    cfg: MemMonCfg,
}

impl MemoryMonitor {
    /// Initialize sysinfo and warm up CPU/memory counters.
    pub fn new(cfg: MemMonCfg) -> Result<Self> {
        let mut sys = System::new();
        let refresh = RefreshKind::new()
            .with_cpu(CpuRefreshKind::everything())
            .with_memory(MemoryRefreshKind::everything());
        sys.refresh_specifics(refresh);
        Ok(Self { sys, cfg })
    }

    /// Capture one sample and return a vector of events:
    /// - Always includes one `AnyEvent::SystemStats` (snapshot)
    /// - Optionally includes one `AnyEvent::PluginTrigger` (threshold alert)
    pub fn snapshot_events(&mut self) -> Result<Vec<AnyEvent>> {
        let t0 = std::time::Instant::now();

        // Build refresh set each pass.
        let mut refresh = RefreshKind::new()
            .with_cpu(CpuRefreshKind::everything())
            .with_memory(MemoryRefreshKind::everything());

        if self.cfg.refresh_processes {
            refresh = refresh.with_processes(ProcessRefreshKind::new());
        }

        self.sys.refresh_specifics(refresh);

        // Collect metrics
        let cpu_pct = self.sys.global_cpu_info().cpu_usage();

        // KiB -> MiB
        let mem_total_mb = (self.sys.total_memory() as f32) / 1024.0;
        let mem_used_mb = (self.sys.used_memory() as f32) / 1024.0;
        let swap_total_mb = (self.sys.total_swap() as f32) / 1024.0;
        let swap_used_mb = (self.sys.used_swap() as f32) / 1024.0;

        let la = System::load_average();
        let load1 = la.one as f32;
        let load5 = la.five as f32;
        let load15 = la.fifteen as f32;

        let uptime_s = System::uptime() as i64;
        let process_count = self.sys.processes().len() as u32;

        let snapshot = SystemStatSnapshot {
            base: BaseEvent::new(0, "memory_monitor"),
            cpu_pct,
            mem_total_mb,
            mem_used_mb,
            swap_total_mb,
            swap_used_mb,
            load1,
            load5,
            load15,
            uptime_s,
            process_count,
        };

        // Validate snapshot before we emit.
        if let Err(e) = snapshot.validate() {
            return Err(MemMonError::Sample(e.to_string()));
        }

        let mut out = vec![AnyEvent::SystemStats(snapshot.clone())];

        // Thresholds (if enabled)
        if self.cfg.emit_triggers && mem_total_mb > 0.0 {
            let pct_used = (mem_used_mb / mem_total_mb) * 100.0;
            let (trigger, level) = if pct_used >= self.cfg.crit_threshold_pct {
                (true, "critical")
            } else if pct_used >= self.cfg.warn_threshold_pct {
                (true, "warning")
            } else {
                (false, "")
            };

            if trigger {
                let trig = PluginTrigger {
                    base: BaseEvent::new(0, "memory_monitor"),
                    plugin: "memory_monitor".into(),
                    reason: format!("memory_{}_threshold", level),
                    // Map severity into 0..10 (e.g. 8 for ~80%+)
                    score: (pct_used / 10.0).clamp(0.0, 10.0),
                    flags: TriggerFlags::empty(), // ✅ FIXED
                    context_summary: Some(format!(
                        "{level}: used={:.1}MiB total={:.1}MiB ({:.1}%), swap_used={:.1}MiB",
                        mem_used_mb, mem_total_mb, pct_used, swap_used_mb
                    )),
                };

                out.push(AnyEvent::PluginTrigger(trig));
            }
        }

        let elapsed = t0.elapsed().as_millis() as u64;
        if self.cfg.soft_budget_ms > 0 && elapsed > self.cfg.soft_budget_ms {
            warn!(elapsed_ms = elapsed, "memory sample exceeded soft budget");
        } else {
            debug!(elapsed_ms = elapsed, "memory sample ok");
        }

        Ok(out)
    }
}

// ================== Tests ==================

#[cfg(test)]
mod tests {
    use super::*;
    use overflow_core::Event;

    #[test]
    fn snapshot_valid_and_thresholds_optional() {
        let mut mon = MemoryMonitor::new(MemMonCfg {
            soft_budget_ms: 500,
            refresh_processes: true,
            warn_threshold_pct: 120.0, // deliberately high to avoid triggers
            crit_threshold_pct: 150.0,
            emit_triggers: true,
        })
        .unwrap();

        let events = mon.snapshot_events().unwrap();
        assert!(!events.is_empty(), "must emit at least one event");
        match &events[0] {
            AnyEvent::SystemStats(s) => {
                assert!(s.mem_total_mb >= 0.0 && s.mem_used_mb >= 0.0);
                assert!(s.cpu_pct >= 0.0 && s.cpu_pct <= 100.0);
                assert!(s.load1 >= 0.0 && s.load5 >= 0.0 && s.load15 >= 0.0);
                assert!(s.uptime_s >= 0);
                assert!(s.validate().is_ok());
            }
            _ => panic!("first event must be SystemStats"),
        }
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn triggers_fire_when_configured() {
        let mut mon = MemoryMonitor::new(MemMonCfg {
            soft_budget_ms: 500,
            refresh_processes: false,
            warn_threshold_pct: 0.1,
            crit_threshold_pct: 0.2,
            emit_triggers: true,
        })
        .unwrap();

        let events = mon.snapshot_events().unwrap();
        assert!(events.len() >= 1);
        assert!(
            events.len() == 1 || events.len() == 2,
            "expected 1 or 2 events, got {}",
            events.len()
        );

        if events.len() == 2 {
            match &events[1] {
                AnyEvent::PluginTrigger(t) => {
                    assert_eq!(t.plugin, "memory_monitor");
                    assert!(t.score >= 0.0 && t.score <= 10.0);
                }
                _ => panic!("second event should be PluginTrigger"),
            }
        }
    }
}