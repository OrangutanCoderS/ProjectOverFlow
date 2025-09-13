use criterion::{criterion_group, criterion_main, Criterion};
use system::battery_monitor::{BatteryCfg, BatteryMonitor};

fn bench_battery_snapshot(c: &mut Criterion) {
    let mon = match BatteryMonitor::new(BatteryCfg { timeout_ms: 300, enrich_ioreg: false }) {
        Ok(m) => m,
        Err(_) => return, // non-macOS
    };

    c.bench_function("battery_snapshot_pmset_only", |b| {
        b.iter(|| {
            let _ = mon.snapshot(); // ignore value; measure latency
        });
    });
}

criterion_group!(benches, bench_battery_snapshot);
criterion_main!(benches);