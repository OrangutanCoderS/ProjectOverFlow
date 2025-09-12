//! System Stats Tests (Phase I)
//! Ensures the system stats monitor works consistently across platforms.

use system::{SystemStatsMonitor, SysStatsCfg};
use overflow_core::{AnyEvent, Event};

#[test]
fn snapshot_succeeds_and_values_reasonable() {
    // Use a soft budget to ensure warnings can trigger but don’t fail
    let mut mon = SystemStatsMonitor::new(SysStatsCfg {
        refresh_processes: true,
        soft_budget_ms: 500,
    })
    .unwrap();

    let evt = mon.snapshot().unwrap();
    match evt {
        AnyEvent::SystemStats(s) => {
            // CPU %
            assert!(s.cpu_pct >= 0.0 && s.cpu_pct <= 100.0);

            // Memory must be non-negative
            assert!(s.mem_total_mb >= 0.0);
            assert!(s.mem_used_mb >= 0.0);
            assert!(s.swap_total_mb >= 0.0);
            assert!(s.swap_used_mb >= 0.0);

            // Sanity: total should not be smaller than used (allowing float noise)
            assert!(s.mem_total_mb + 1.0 >= s.mem_used_mb);
            assert!(s.swap_total_mb + 1.0 >= s.swap_used_mb);

            // Load averages non-negative
            assert!(s.load1 >= 0.0 && s.load5 >= 0.0 && s.load15 >= 0.0);

            // Uptime non-negative
            assert!(s.uptime_s >= 0);

            // Process count not absurdly large
            assert!(s.process_count < 1_000_000);

            // Validation must succeed (requires Event in scope)
            assert!(s.validate().is_ok());
        }
        _ => panic!("unexpected event kind"),
    }
}