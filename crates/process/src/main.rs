use process::{ProcessMonitor, ProcMonCfg};
use std::{thread, time::Duration};

fn main() {
    let mut mon = ProcessMonitor::new(ProcMonCfg::default())
        .expect("Failed to init ProcessMonitor");

    loop {
        // Capture snapshot
        let snap = mon.capture().expect("Failed to capture snapshot");

        // Clear screen (ANSI escape codes)
        print!("\x1B[2J\x1B[1;1H");

        // Header
        println!("{:<8} {:<6} {:<8} {:<8} {}", "PID", "CPU%", "MEM(MB)", "USER", "CMD");
        println!("{}", "-".repeat(80));

        // Sort by CPU descending
        let mut procs: Vec<_> = snap.map.values().collect();
        procs.sort_by(|a, b| b.cpu_pct.partial_cmp(&a.cpu_pct).unwrap());

        // Print top 15
        for info in procs.into_iter().take(15) {
            println!(
                "{:<8} {:<6.1} {:<8.1} {:<8} {}",
                info.base.pid, info.cpu_pct, info.mem_mb, info.user, info.name
            );
        }

        // Sleep before next refresh
        thread::sleep(Duration::from_secs(2));
    }
}