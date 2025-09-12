use criterion::{criterion_group, criterion_main, Criterion};
use system::{SystemCfg, SystemMonitor};

fn bench_capture(c: &mut Criterion) {
    let mut mon = SystemMonitor::new(SystemCfg::default()).unwrap();

    c.bench_function("system_capture", |b| {
        b.iter(|| {
            let _ = mon.capture().unwrap();
        })
    });
}

criterion_group!(benches, bench_capture);
criterion_main!(benches);