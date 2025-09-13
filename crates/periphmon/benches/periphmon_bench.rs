use criterion::{criterion_group, criterion_main, Criterion};
use periphmon::{poll_peripherals, PeriphConfig};

fn bench_poll(c: &mut Criterion) {
    let cfg = PeriphConfig::default();
    c.bench_function("periphmon_poll_small", |b| {
        b.iter(|| poll_peripherals(&cfg).unwrap())
    });
}

criterion_group!(benches, bench_poll);
criterion_main!(benches);
