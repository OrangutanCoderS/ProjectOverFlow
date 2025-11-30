use std::{thread, time::Duration};

use anyhow::Result;
use overflow_core::{AnyEvent, BaseEvent, ThermalEvent};

// Correct module paths for ALL monitors
use system::cpu_tracker::{CpuTracker, CpuCfg};
use system::memory_monitor::{MemoryMonitor, MemMonCfg};
use system::battery_monitor::{BatteryMonitor, BatteryCfg};
use system::gpu_tracker::{GpuTracker, GpuCfg};
use system::thermal_logger::{ThermalLogger, ThermalCfg};

fn main() -> Result<()> {
    // Initialize CPU + Memory (always supported)
    let mut cpu = CpuTracker::new(CpuCfg::default())?;
    let mut mem = MemoryMonitor::new(MemMonCfg::default())?;

    // Optional modules — .ok() to gracefully ignore unsupported OS/hardware
    let mut gpu = GpuTracker::new(GpuCfg::default()).ok();
    let mut batt = BatteryMonitor::new(BatteryCfg::default()).ok();
    let mut therm = ThermalLogger::new(ThermalCfg::default()).ok();

    loop {
        // CPU → CpuEvent
        if let Ok(evt) = cpu.capture() {
            println!("{}", serde_json::to_string(&AnyEvent::Cpu(evt))?);
        }

        // Memory → Vec<AnyEvent>
        if let Ok(events) = mem.snapshot_events() {
            for evt in events {
                println!("{}", serde_json::to_string(&evt)?);
            }
        }

        // GPU → AnyEvent::Gpu
        if let Some(ref mut g) = gpu {
            if let Ok(evt) = g.capture() {
                println!("{}", serde_json::to_string(&evt)?);
            }
        }

        // Battery → AnyEvent::Battery
        if let Some(ref mut b) = batt {
            if let Ok(evt) = b.snapshot() {
                println!("{}", serde_json::to_string(&evt)?);
            }
        }

        // Thermal → ThermalSnapshot → convert → AnyEvent::Thermal
        if let Some(ref mut t) = therm {
            if let Ok(snapshot) = t.capture() {
                if snapshot.is_meaningful() {
                    let base = BaseEvent::new(0, "thermal_logger");
                    let thermal_event = ThermalEvent {
                        base,
                        cpu_temp_c: snapshot.cpu_temp_c,
                        gpu_temp_c: snapshot.gpu_temp_c,
                        skin_temp_c: snapshot.soc_temp_c,
                        package_power_w: None,
                        notes: snapshot.throttling.map(|t| format!("Throttling detected: {}", t)),
                    };
                    println!("{}", serde_json::to_string(&AnyEvent::Thermal(thermal_event))?);
                }
            }
        }

        thread::sleep(Duration::from_millis(1000));
    }
}