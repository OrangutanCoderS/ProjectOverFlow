//! Module 7 — CPU Tracker (Phase I)
//! Cross-platform CPU usage snapshots using sysinfo.
//! Emits structured CpuEvent via overflow_core.

use overflow_core::{AnyEvent, BaseEvent, CpuEvent, Event, TimestampMs};
use overflow_utils::unix_time_ms;
use serde::{Deserialize, Serialize};
use sysinfo::{CpuRefreshKind, RefreshKind, System};
use thiserror::Error;
use tracing::{debug, warn};

#[derive(Debug, Error)]
pub enum CpuError {
    #[error("system init: {0}")]
    Init(String),
    #[error("snapshot: {0}")]
    Snapshot(String),
}

pub type Result<T> = std::result::Result<T, CpuError>;

/// Config for CPU tracker.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CpuCfg {
    /// Warn if refresh takes longer than this many ms.
    pub soft_budget_ms: u64,
}

impl Default for CpuCfg {
    fn default() -> Self {
        Self { soft_budget_ms: 50 }
    }
}

/// CPU tracker that captures CpuEvent.
pub struct CpuTracker {
    sys: System,
    cfg: CpuCfg,
}

impl CpuTracker {
    pub fn new(cfg: CpuCfg) -> Result<Self> {
        let mut sys = System::new();
        let refresh = RefreshKind::new().with_cpu(CpuRefreshKind::everything());
        sys.refresh_specifics(refresh);
        Ok(Self { sys, cfg })
    }

    /// Capture one CPU snapshot as a CpuEvent.
    pub fn capture(&mut self) -> Result<CpuEvent> {
        let t0 = std::time::Instant::now();
        self.sys.refresh_cpu_specifics(CpuRefreshKind::everything());

        let global_usage = self.sys.global_cpu_info().cpu_usage();
        let per_core: Vec<f32> = self.sys.cpus().iter().map(|c| c.cpu_usage()).collect();

        let evt = CpuEvent {
            base: BaseEvent::new(0, "cpu_tracker"),
            global_usage,
            per_core,
        };

        // Validate model for safety
        evt.validate()
            .map_err(|e| CpuError::Snapshot(e.to_string()))?;

        let elapsed = t0.elapsed().as_millis() as u64;
        if elapsed > self.cfg.soft_budget_ms {
            warn!(elapsed_ms = elapsed, "cpu snapshot exceeded soft budget");
        } else {
            debug!(elapsed_ms = elapsed, "cpu snapshot ok");
        }

        Ok(evt)
    }

    /// Wrap CpuEvent in AnyEvent for uniform dispatch.
    pub fn capture_any(&mut self) -> Result<AnyEvent> {
        let evt = self.capture()?;
        Ok(AnyEvent::Cpu(evt))
    }
}
