//! Module 6 — System Stats (Phase I)
//! Captures CPU %, memory/swap, load averages, uptime, process count and emits a unified event.

use overflow_core::{AnyEvent, BaseEvent, Event, SystemStatSnapshot};
use sysinfo::{CpuRefreshKind, MemoryRefreshKind, ProcessRefreshKind, RefreshKind, System};
use thiserror::Error;
use tracing::{debug, warn};

#[derive(Debug, Error)]
pub enum SysStatsError {
    #[error("init: {0}")]
    Init(String),
    #[error("sample: {0}")]
    Sample(String),
}

pub type Result<T> = std::result::Result<T, SysStatsError>;

/// Minimal configuration for sampling cadence, soft budget, etc., if needed later.
#[derive(Debug, Clone, Default)]
pub struct SysStatsCfg {
    /// If true, include a quick processes refresh so `process_count` is fresh.
    pub refresh_processes: bool,
    /// Soft time budget for one sample in ms (warn if exceeded).
    pub soft_budget_ms: u64,
}

pub struct SystemStatsMonitor {
    sys: System,
    cfg: SysStatsCfg,
}

impl SystemStatsMonitor {
    pub fn new(cfg: SysStatsCfg) -> Result<Self> {
        // Initialize once; we’ll refresh specific parts on each tick.
        let mut sys = System::new();

        // Warm-up: CPU + memory (system uptime/load are static)
        let refresh = RefreshKind::new()
            .with_cpu(CpuRefreshKind::everything())
            .with_memory(MemoryRefreshKind::everything());
        sys.refresh_specifics(refresh);

        Ok(Self { sys, cfg })
    }

    /// Capture one system snapshot and return as AnyEvent.
    pub fn snapshot(&mut self) -> Result<AnyEvent> {
        let t0 = std::time::Instant::now();

        // Always refresh CPU + memory each sample.
        let mut refresh = RefreshKind::new()
            .with_cpu(CpuRefreshKind::everything())
            .with_memory(MemoryRefreshKind::everything());

        // Optionally refresh processes to update process_count
        if self.cfg.refresh_processes {
            refresh = refresh.with_processes(ProcessRefreshKind::new());
        }

        self.sys.refresh_specifics(refresh);

        // CPU usage: global % (0..=100)
        let cpu_pct = self.sys.global_cpu_info().cpu_usage();

        // memory and swap in KiB -> MiB
        let mem_total_mb = (self.sys.total_memory() as f32) / 1024.0;
        let mem_used_mb = (self.sys.used_memory() as f32) / 1024.0;
        let swap_total_mb = (self.sys.total_swap() as f32) / 1024.0;
        let swap_used_mb = (self.sys.used_swap() as f32) / 1024.0;

        // Load averages (1, 5, 15)
        let la = System::load_average();
        let load1 = la.one as f32;
        let load5 = la.five as f32;
        let load15 = la.fifteen as f32;

        // Uptime seconds and process count
        let uptime_s = System::uptime() as i64;
        let process_count = self.sys.processes().len() as u32;

        let evt = SystemStatSnapshot {
            base: BaseEvent::new(0, "system_stats"),
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

        // Validate for safety (Event trait in scope)
        if let Err(e) = evt.validate() {
            return Err(SysStatsError::Sample(e.to_string()));
        }

        let elapsed = t0.elapsed().as_millis() as u64;
        if self.cfg.soft_budget_ms > 0 && elapsed > self.cfg.soft_budget_ms {
            warn!(
                elapsed_ms = elapsed,
                "system stats sample exceeded soft budget"
            );
        } else {
            debug!(elapsed_ms = elapsed, "system stats sample ok");
        }

        Ok(AnyEvent::SystemStats(evt))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_succeeds_and_values_reasonable() {
        let mut mon = SystemStatsMonitor::new(SysStatsCfg {
            refresh_processes: true,
            soft_budget_ms: 500,
        })
        .unwrap();

        let evt = mon.snapshot().unwrap();
        match evt {
            AnyEvent::SystemStats(s) => {
                assert!(s.cpu_pct >= 0.0 && s.cpu_pct <= 100.0);
                assert!(s.mem_total_mb >= 0.0);
                assert!(s.mem_used_mb >= 0.0);
                assert!(s.swap_total_mb >= 0.0);
                assert!(s.swap_used_mb >= 0.0);
                assert!(s.load1 >= 0.0 && s.load5 >= 0.0 && s.load15 >= 0.0);
                assert!(s.uptime_s >= 0);
                assert!(s.process_count < 1_000_000);
                assert!(s.validate().is_ok());
            }
            _ => panic!("unexpected event kind"),
        }
    }
}
pub mod cpu_tracker;

pub mod memory_monitor;

pub mod gpu_tracker;

pub mod battery_monitor;

pub use battery_monitor::{BatteryCfg, BatteryError, BatteryMonitor};

pub mod thermal_logger;

