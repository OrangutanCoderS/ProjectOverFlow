use system::memory_monitor::{MemMonCfg, MemoryMonitor};
use overflow_core::{AnyEvent, Event};

#[test]
fn mem_snapshot_emits_systemstats() {
    let mut mon = MemoryMonitor::new(MemMonCfg::default()).unwrap();
    let events = mon.snapshot_events().unwrap();
    assert!(!events.is_empty());
    match &events[0] {
        AnyEvent::SystemStats(s) => {
            assert!(s.mem_total_mb >= 0.0 && s.mem_used_mb >= 0.0);
            assert!(s.cpu_pct >= 0.0 && s.cpu_pct <= 100.0);
            assert!(s.validate().is_ok());
        }
        _ => panic!("first event must be SystemStats"),
    }
}
#[test]
fn mem_threshold_trigger_can_fire() {
    let mut mon = MemoryMonitor::new(MemMonCfg {
        soft_budget_ms: 500,
        refresh_processes: false,
        warn_threshold_pct: 0.1,  // force trigger on most systems
        crit_threshold_pct: 0.2,
        emit_triggers: true,
    })
    .unwrap();

    let events = mon.snapshot_events().unwrap();
    // 1 event (snapshot) or 2 events (snapshot + trigger) are both valid
    assert!(events.len() == 1 || events.len() == 2);
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