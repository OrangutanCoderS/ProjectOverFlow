use system::gpu_tracker::{GpuCfg, GpuTracker};
use overflow_core::{AnyEvent, Event, GpuEvent};

#[test]
fn gpu_parsers_compile_and_tracker_constructs() {
    // Tracker should construct on macOS; on other OS it should error (we skip).
    let tracker = GpuTracker::new(GpuCfg::default());
    if tracker.is_err() {
        return; // skip on non-macOS CI
    }

    let tracker = tracker.unwrap();

    // We don't assert actual metrics (they depend on host/permissions),
    // but we ensure the call returns a well-formed event or a clean error.
    let res = tracker.capture();
    match res {
        Ok(AnyEvent::Gpu(e)) => {
            // Ensure GPU name string exists (may be "Unknown GPU").
            assert!(!e.name.is_empty());
            // Validation should succeed.
            assert!(e.validate().is_ok());
        }
        Ok(_) => panic!("unexpected event type"),
        Err(_) => {
            // Acceptable if environment denies powermetrics & system_profiler
            // (sandbox/CI); no panic — we tested parsing separately.
        }
    }
}