use system::{SystemCfg, SystemMonitor, SystemSnapshot};

#[test]
fn snapshot_captures_system() {
    let mut mon = SystemMonitor::new(SystemCfg {
        include_disks: true,
        soft_budget_ms: 500,
    })
    .unwrap();

    let snap: SystemSnapshot = mon.capture().unwrap();

    // Sanity checks
    assert!(snap.ts_ms > 0, "timestamp must be positive");
    assert!(snap.cpu_pct >= 0.0, "CPU % must be non-negative");
    assert!(
        snap.mem_total_mib >= snap.mem_used_mib,
        "Total memory must always be >= used memory"
    );
}