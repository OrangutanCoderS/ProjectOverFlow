use criterion::{black_box, Criterion, criterion_group, criterion_main};
use cloudsync::{poll_cloudsync, CloudSyncConfig};

pub fn cloudsync_bench(c: &mut Criterion) {
    let cfg = CloudSyncConfig::default();
    c.bench_function("cloudsync_poll_small", |b| {
        b.iter(|| poll_cloudsync(black_box(&cfg)).unwrap())
    });
}

criterion_group!(benches, cloudsync_bench);
criterion_main!(benches);