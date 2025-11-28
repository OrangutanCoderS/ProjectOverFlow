use autonomous_runtime::{
    AutonomousRuntime, NullActionSink, NullEventSource, RuntimeClock, RuntimeCommand,
    RuntimeConfig, RuntimeTickStats, RuntimeStatus,
};
use std::time::Duration;

/// Test clock that increments a counter instead of sleeping.
#[derive(Default)]
struct TestClock {
    pub sleeps: u64,
}

impl RuntimeClock for TestClock {
    fn sleep(&mut self, _duration: Duration) {
        self.sleeps += 1;
    }
}

#[test]
fn config_validation_rejects_zero_values() {
    let cfg = RuntimeConfig {
        tick_interval_ms: 0,
        ..RuntimeConfig::default()
    };

    assert!(cfg.validate().is_err());

    let cfg = RuntimeConfig {
        max_events_per_tick: 0,
        ..RuntimeConfig::default()
    };
    assert!(cfg.validate().is_err());

    let cfg = RuntimeConfig {
        flush_interval_ticks: 0,
        ..RuntimeConfig::default()
    };
    assert!(cfg.validate().is_err());
}

#[test]
fn runtime_initializes_and_runs_single_tick() {
    let cfg = RuntimeConfig::default();
    let source = NullEventSource;
    let sink = NullActionSink;
    let clock = TestClock::default();

    let mut rt = AutonomousRuntime::new(cfg, source, sink, clock)
        .expect("runtime should initialize");

    assert_eq!(rt.state().status, RuntimeStatus::Initialized);

    rt.apply_command(RuntimeCommand::Start)
        .expect("start command should succeed");

    assert_eq!(rt.state().status, RuntimeStatus::Running);

    let stats = rt.tick_once().expect("tick should succeed");

    assert_eq!(stats.tick_number, 1);
    assert_eq!(stats.events_polled, 0);
    assert_eq!(stats.events_processed, 0);
}

#[test]
fn runtime_requires_running_status_for_tick() {
    let cfg = RuntimeConfig::default();
    let source = NullEventSource;
    let sink = NullActionSink;
    let clock = TestClock::default();

    let mut rt = AutonomousRuntime::new(cfg, source, sink, clock)
        .expect("runtime should initialize");

    let err = rt.tick_once().unwrap_err();
    match err {
        autonomous_runtime::RuntimeError::NotRunning(status) => {
            assert_eq!(status, RuntimeStatus::Initialized);
        }
        other => panic!("unexpected error: {other:?}"),
    }
}