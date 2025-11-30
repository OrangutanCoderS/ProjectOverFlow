use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

use anyhow::Result;
use overflow_core::{AnyEvent, TelemetrySender};
use crate::{
    SystemStatsMonitor, SysStatsCfg,
    BatteryMonitor, BatteryCfg,
    gpu_tracker::{GpuTracker, GpuCfg},
    thermal_logger::{ThermalLogger, ThermalCfg},
};

// === Subsystem Trait ===
// Matches daemon expectations
pub trait Subsystem: Send + Sync {
    fn name(&self) -> &'static str;
    fn start(&self, tx: TelemetrySender) -> Result<()>;
    fn stop(&self) -> Result<()>;
}

pub struct SystemSubsystem {
    running: Arc<AtomicBool>,
}

impl SystemSubsystem {
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl Subsystem for SystemSubsystem {
    fn name(&self) -> &'static str {
        "system"
    }

    fn start(&self, tx: TelemetrySender) -> Result<()> {
        if self.running.swap(true, Ordering::SeqCst) {
            return Ok(());
        }

        let running = self.running.clone();

        // ========== CPU / MEM THREAD ==========
        {
            let tx = tx.clone();
            let running = running.clone();

            thread::spawn(move || {
                let mut sysmon = SystemStatsMonitor::new(SysStatsCfg {
                    refresh_processes: true,
                    soft_budget_ms: 300,
                }).expect("system stats init failed");

                while running.load(Ordering::SeqCst) {
                    if let Ok(evt) = sysmon.snapshot() {
                        let _ = tx.send(evt);
                    }
                    thread::sleep(Duration::from_millis(1200));
                }
            });
        }

        // ========== BATTERY THREAD ==========
        {
            let tx = tx.clone();
            let running = running.clone();
            thread::spawn(move || {
                let mut batt = match BatteryMonitor::new(BatteryCfg::default()) {
                    Ok(b) => b,
                    Err(_) => return, // unsupported on this platform
                };

                while running.load(Ordering::SeqCst) {
                    if let Ok(evt) = batt.snapshot() {
                        let _ = tx.send(evt);
                    }
                    thread::sleep(Duration::from_millis(5000));
                }
            });
        }

        // ========== GPU THREAD ==========
        {
            let tx = tx.clone();
            let running = running.clone();

            thread::spawn(move || {
                let mut gpu = match GpuTracker::new(GpuCfg::default()) {
                    Ok(g) => g,
                    Err(_) => return,
                };

                while running.load(Ordering::SeqCst) {
                    if let Ok(evt) = gpu.capture() {
                        let _ = tx.send(evt);
                    }
                    thread::sleep(Duration::from_millis(2000));
                }
            });
        }

        // ========== THERMAL THREAD ==========
        {
            let tx = tx.clone();
            let running = running.clone();

            thread::spawn(move || {
                let therm = match ThermalLogger::new(ThermalCfg::default()) {
                    Ok(t) => t,
                    Err(_) => return,
                };

                while running.load(Ordering::SeqCst) {
                    if let Ok(snapshot) = therm.capture() {
                        if snapshot.is_meaningful() {
                            let evt = AnyEvent::Thermal(snapshot);
                            let _ = tx.send(evt);
                        }
                    }
                    thread::sleep(Duration::from_millis(3000));
                }
            });
        }

        Ok(())
    }

    fn stop(&self) -> Result<()> {
        self.running.store(false, Ordering::SeqCst);
        Ok(())
    }
}