use process::{ProcMonCfg, ProcessMonitor};

#[test]
fn smoke_tick_returns_events() {
    let mut mon = ProcessMonitor::new(ProcMonCfg {
        emit_deltas: false,
        soft_budget_ms: 500,
    })
    .unwrap();
    let evts = mon.tick().unwrap();
    // On any live system this should be >0; if not, at least we didn't panic.
    assert!(evts.len() >= 1);
}
