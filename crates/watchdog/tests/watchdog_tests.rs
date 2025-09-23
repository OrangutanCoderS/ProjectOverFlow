use watchdog::{
    Watchdog, WatchdogConfig, RegisterSpec, UnitKind, RestartPolicy, WatchdogError
};
use pretty_assertions::assert_eq;
use std::{sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
    thread,
};

#[test]
fn register_and_heartbeat() {
    let mut wd = Watchdog::new(WatchdogConfig { poll_interval: Duration::from_millis(20), batch: 64 });
    wd.start().unwrap();

    let beats = Arc::new(AtomicUsize::new(0));
    let beats_clone = beats.clone();

    let spec = RegisterSpec {
        id: "unit-a".into(),
        kind: UnitKind::ThreadFn,
        timeout: Duration::from_millis(120),
        policy: RestartPolicy::default(),
        restart_cb: Arc::new(move || {
            // simulate restart by "resetting" heartbeat count
            beats_clone.store(0, Ordering::Relaxed);
            Ok(())
        }),
    };
    wd.register(spec).unwrap();

    // Send some heartbeats
    for _ in 0..3 {
        wd.heartbeat("unit-a").unwrap();
        beats.fetch_add(1, Ordering::Relaxed);
        thread::sleep(Duration::from_millis(30));
    }

    assert!(beats.load(Ordering::Relaxed) >= 3);
    wd.stop();
}

#[test]
fn restart_on_missed_heartbeat() {
    let mut wd = Watchdog::new(WatchdogConfig { poll_interval: Duration::from_millis(25), batch: 256 });
    wd.start().unwrap();

    let restarts = Arc::new(AtomicUsize::new(0));
    let restarts_clone = restarts.clone();

    let spec = RegisterSpec {
        id: "unit-b".into(),
        kind: UnitKind::ThreadFn,
        timeout: Duration::from_millis(80),
        policy: RestartPolicy {
            max_restarts: 3,
            base_backoff: Duration::from_millis(10),
            max_backoff: Duration::from_millis(30),
        },
        restart_cb: Arc::new(move || {
            restarts_clone.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }),
    };
    wd.register(spec).unwrap();

    // No heartbeat -> should restart at least once
    thread::sleep(Duration::from_millis(250));
    let cnt = restarts.load(Ordering::SeqCst);
    assert!(cnt >= 1, "expected at least 1 restart, got {cnt}");

    wd.stop();
}

#[test]
fn duplicate_registration_rejected() {
    let wd = Watchdog::new(WatchdogConfig::default());
    let cb = Arc::new(|| Ok::<(), WatchdogError>(()));
    let spec = RegisterSpec {
        id: "dup".into(),
        kind: UnitKind::ThreadFn,
        timeout: Duration::from_millis(100),
        policy: RestartPolicy::default(),
        restart_cb: cb.clone(),
    };
    wd.register(spec).unwrap();
    let e = wd.register(RegisterSpec {
        id: "dup".into(),
        kind: UnitKind::ThreadFn,
        timeout: Duration::from_millis(100),
        policy: RestartPolicy::default(),
        restart_cb: cb,
    }).unwrap_err();

    let msg = format!("{e}");
    assert!(msg.contains("unit already exists"));
}
