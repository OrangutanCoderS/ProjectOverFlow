use std::{thread, time::Duration};

use overflow_core::AnyEvent;
use system::{
    SystemStatsMonitor, SysStatsCfg,
    BatteryMonitor, BatteryCfg,
};

// GPU + Thermal modules are in submodules
use system::gpu_tracker::{GpuTracker, GpuCfg};
use system::thermal_logger::{ThermalLogger, ThermalCfg};

fn main() {
    let verbose = std::env::args().any(|a| a == "--verbose");

    // System-wide stats
    let mut sysmon = SystemStatsMonitor::new(SysStatsCfg {
        refresh_processes: true,
        soft_budget_ms: 500,
    }).expect("Failed to init SystemStatsMonitor");

    // Optional modules (gracefully degrade on unsupported OS/hardware)
    let mut battmon = BatteryMonitor::new(BatteryCfg::default()).ok();
    let mut gputr   = GpuTracker::new(GpuCfg::default()).ok();
    let     therm   = ThermalLogger::new(ThermalCfg::default()).ok();

    loop {
        // ===== System Stats (CPU, Mem, Swap, Load, Uptime, Proc Count) =====
        let evt = sysmon.snapshot().expect("Failed to capture system stats");

        // Clear screen
        print!("\x1B[2J\x1B[1;1H");

        if let AnyEvent::SystemStats(s) = evt {
            println!("=== ShadowTrace System Monitor {} ===",
                if verbose { "(Verbose)" } else { "(Minimal)" });
            println!("Uptime: {}s | Processes: {}", s.uptime_s, s.process_count);
            println!();

            // CPU + Memory (always available here)
            let mem_pct = if s.mem_total_mb > 0.0 {
                (s.mem_used_mb / s.mem_total_mb) * 100.0
            } else { 0.0 };

            if verbose {
                println!("CPU Usage (%) : {:>6.2}", s.cpu_pct);
                println!("Memory Total  : {:>8.2} MB", s.mem_total_mb);
                println!("Memory Used   : {:>8.2} MB ({:>5.2}%)", s.mem_used_mb, mem_pct);
                println!("Swap Total    : {:>8.2} MB", s.swap_total_mb);
                println!("Swap Used     : {:>8.2} MB", s.swap_used_mb);
                println!("Load Average  : 1m={:.2}, 5m={:.2}, 15m={:.2}", s.load1, s.load5, s.load15);
            } else {
                println!("CPU Usage : {:>5.1}%", s.cpu_pct);
                println!("Memory    : {:>6.1}/{:<6.1} MB ({:>4.1}%)",
                         s.mem_used_mb, s.mem_total_mb, mem_pct);
                println!("Swap      : {:>6.1}/{:<6.1} MB", s.swap_used_mb, s.swap_total_mb);
                println!("Load Avg  : {:>4.2}, {:>4.2}, {:>4.2}", s.load1, s.load5, s.load15);
            }
        }

        // ===== Battery (AnyEvent::Battery) =====
        if let Some(ref mut bm) = battmon {
            match bm.snapshot() {
                Ok(AnyEvent::Battery(b)) => {
                    if verbose {
                        println!("\nBattery:");
                        println!("  Percent       : {:>5.2}%", b.percentage);
                        println!("  Charging      : {}", if b.charging { "Yes" } else { "No" });
                        if let Some(cc) = b.cycle_count { println!("  Cycle Count   : {}", cc); }
                        if let Some(t)  = b.temperature_c { println!("  Temp (°C)     : {:.1}", t); }
                        if let Some(v)  = b.voltage_mv { println!("  Voltage (mV)  : {}", v); }
                        if let Some(h)  = b.health.as_deref() { println!("  Health        : {}", h); }
                        if let Some(min)= b.time_remaining_min { println!("  Time Left (m) : {}", min); }
                    } else {
                        println!("\nBattery   : {:>5.1}% | {}",
                                 b.percentage,
                                 if b.charging { "Charging" } else { "Discharging" });
                    }
                }
                _ => { /* unsupported or temporarily unavailable */ }
            }
        }

        // ===== GPU (AnyEvent::Gpu) =====
        if let Some(ref mut gt) = gputr {
            if let Ok(AnyEvent::Gpu(g)) = gt.capture() {
                if verbose {
                    println!("\nGPU:");
                    println!("  Name          : {}", g.name);
                    println!("  Usage (%)     : {:.1}", g.usage_pct);
                    println!("  Temp (°C)     : {:.1}", g.temperature_c);
                    println!("  Mem Used/Total: {:.1}/{:.1} MB", g.mem_used_mb, g.mem_total_mb);
                } else {
                    println!("GPU       : {:>4.1}% | {} | {:.1}/{:.1} MB",
                             g.usage_pct, g.name, g.mem_used_mb, g.mem_total_mb);
                }
            }
        }

        // ===== Thermal (local ThermalSnapshot, not AnyEvent) =====
        if let Some(ref tl) = therm {
            if let Ok(snap) = tl.capture() {
                // Only print if we have something meaningful
                if snap.cpu_temp_c.is_some() || snap.gpu_temp_c.is_some() || snap.soc_temp_c.is_some() {
                    if verbose {
                        println!("\nThermals:");
                        if let Some(t) = snap.cpu_temp_c { println!("  CPU (°C)      : {:.1}", t); }
                        if let Some(t) = snap.gpu_temp_c { println!("  GPU (°C)      : {:.1}", t); }
                        if let Some(t) = snap.soc_temp_c { println!("  SoC (°C)      : {:.1}", t); }
                        if let Some(th) = snap.throttling { println!("  Throttling    : {}", if th { "Yes" } else { "No" }); }
                    } else {
                        let cpu = snap.cpu_temp_c.map(|t| format!("CPU {:.1}°C", t));
                        let gpu = snap.gpu_temp_c.map(|t| format!("GPU {:.1}°C", t));
                        let soc = snap.soc_temp_c.map(|t| format!("SoC {:.1}°C", t));
                        let parts = [cpu, gpu, soc].into_iter().flatten().collect::<Vec<_>>().join(" | ");
                        if !parts.is_empty() {
                            println!("Thermals : {}", parts);
                        }
                    }
                }
            }
        }

        println!("\n(Updating every 2 seconds)  Use --verbose for more detail.");
        thread::sleep(Duration::from_secs(2));
    }
}