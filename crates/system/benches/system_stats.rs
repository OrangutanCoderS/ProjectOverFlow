use criterion::{criterion_group, criterion_main, Criterion};
use system::{SysStatsCfg, SystemStatsMonitor};

fn bench_capture(c: &mut Criterion) {
    let mut mon = SystemStatsMonitor::new(SysStatsCfg::default()).unwrap();

    c.bench_function("system_stats_capture", |b| {
        b.iter(|| {
            let _ = mon.snapshot().unwrap();
        })
    });
}

criterion_group!(benches, bench_capture);
criterion_main!(benches);
