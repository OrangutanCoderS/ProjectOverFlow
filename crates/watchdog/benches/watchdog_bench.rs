use criterion::{criterion_group, criterion_main, Criterion, black_box};
use std::{sync::Arc, time::Duration, thread};
use watchdog::{Watchdog, WatchdogConfig, RegisterSpec, UnitKind, RestartPolicy, WatchdogError};

fn bench_heartbeats(c: &mut Criterion) {
    let mut wd = Watchdog::new(WatchdogConfig {
        poll_interval: Duration::from_millis(10),
        batch: 512,
    });
    wd.start().unwrap();

    let cb = Arc::new(|| Ok::<(), WatchdogError>(()));
    for i in 0..100 {
        wd.register(RegisterSpec {
            id: format!("u{i}"),
            kind: UnitKind::ThreadFn,
            timeout: Duration::from_millis(1000), // was causing timeout when small
            policy: RestartPolicy::default(),
            restart_cb: cb.clone(),
        }).unwrap();
    }

    c.bench_function("heartbeat_10k", |b| {
        b.iter(|| {
            for i in 0..100 {
                wd.heartbeat(&format!("u{i}")).unwrap();
            }
        })
    });

    wd.stop();
}

fn bench_monitor_loop(c: &mut Criterion) {
    let mut wd = Watchdog::new(WatchdogConfig {
        poll_interval: Duration::from_millis(5),
        batch: 256,
    });
    wd.start().unwrap();
    let cb = Arc::new(|| Ok::<(), WatchdogError>(()));

    for i in 0..200 {
        wd.register(RegisterSpec {
            id: format!("m{i}"),
            kind: UnitKind::ThreadFn,
            timeout: Duration::from_millis(100), //  bumped from 20 → 100
            policy: RestartPolicy::default(),
            restart_cb: cb.clone(),
        }).unwrap();
    }

    c.bench_function("monitor_loop_200", |b| {
        b.iter(|| {
            // Let the loop run a few ticks
            thread::sleep(Duration::from_millis(30));
            black_box(());
        })
    });

    wd.stop();
}

criterion_group!(benches, bench_heartbeats, bench_monitor_loop);
criterion_main!(benches);