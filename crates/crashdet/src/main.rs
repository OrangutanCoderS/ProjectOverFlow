// crates/crashdet/src/main.rs

use anyhow::Result;
use crashdet::{CrashConfig, spawn_crash_watcher};

fn main() -> Result<()> {
    // Use default config:
    // - macOS: ~/Library/Logs/DiagnosticReports, /Library/Logs/DiagnosticReports, /Library/Logs/panic.log
    // - poll_interval_ms = 5000
    let mut cfg = CrashConfig::default();

    // If you want more aggressive polling, uncomment:
    // cfg.poll_interval_ms = 2000;

    let rx = spawn_crash_watcher(cfg);

    println!("Starting crash detector...");
    println!("Watching macOS DiagnosticReports and panic.log (where available).\n");

    // Block and print events as they arrive
    loop {
        match rx.recv() {
            Ok(ev) => {
                // One line per crash/panic
                println!(
                    "[{}] severity={} process={} pid={:?} reason=\"{}\" uptime_at_crash={:?}",
                    ev.timestamp,
                    ev.severity,
                    ev.process,
                    ev.pid,
                    ev.crash_reason,
                    ev.uptime_at_crash_sec,
                );
            }
            Err(err) => {
                eprintln!("crashdet: watcher channel closed: {err}");
                break;
            }
        }
    }

    Ok(())
}