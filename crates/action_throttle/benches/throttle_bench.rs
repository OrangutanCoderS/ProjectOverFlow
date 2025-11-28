use criterion::{criterion_group, criterion_main, Criterion};
use action_throttle::{ActionThrottleManager, ThrottleMode, ThrottleRequest};

fn bench_throttle(c: &mut Criterion) {
    c.bench_function("cpu_throttle_50%", |b| {
        b.iter(|| {
            let req = ThrottleRequest {
                pid: std::process::id() as i32,
                mode: ThrottleMode::CPU,
                intensity: 0.5,
                duration_secs: Some(1),
                context: Some("bench".into()),
            };
            ActionThrottleManager::execute(req).unwrap();
        });
    });
}

criterion_group!(benches, bench_throttle);
criterion_main!(benches);