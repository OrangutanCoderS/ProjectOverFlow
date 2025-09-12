use system::cpu_tracker::{CpuCfg, CpuTracker};
use overflow_core::{Event, CpuEvent, AnyEvent};

#[test]
fn cpu_snapshot_direct_event() {
    let mut tracker = CpuTracker::new(CpuCfg::default()).unwrap();
    let snap: CpuEvent = tracker.capture().unwrap();

    // Global usage sanity
    assert!(snap.global_usage >= 0.0 && snap.global_usage <= 100.0);

    // Must report at least one core
    assert!(!snap.per_core.is_empty());

    // Each per-core usage within sane bounds
    for (i, &val) in snap.per_core.iter().enumerate() {
        assert!(
            val >= 0.0 && val <= 100.0,
            "core {i} usage out of bounds: {val}"
        );
    }

    // Model validation should succeed
    assert!(snap.validate().is_ok());
}

#[test]
fn cpu_snapshot_wrapped_anyevent() {
    let mut tracker = CpuTracker::new(CpuCfg::default()).unwrap();
    let evt = tracker.capture_any().unwrap();

    match evt {
        AnyEvent::Cpu(cpu) => {
            assert!(cpu.global_usage >= 0.0 && cpu.global_usage <= 100.0);
            assert!(!cpu.per_core.is_empty());
            assert!(cpu.validate().is_ok());
        }
        _ => panic!("Expected AnyEvent::Cpu, got something else"),
    }
}