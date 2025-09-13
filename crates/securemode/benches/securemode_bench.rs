use criterion::{criterion_group, criterion_main, Criterion};
use securemode::{evaluate_secure_mode, SecureConfig};

fn bench_eval(c: &mut Criterion) {
    let cfg = SecureConfig::default();
    c.bench_function("securemode_eval", |b| {
        b.iter(|| evaluate_secure_mode(&cfg, 0.55, 3, "bench").unwrap())
    });
}

criterion_group!(benches, bench_eval);
criterion_main!(benches);
