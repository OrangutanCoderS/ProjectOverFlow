use criterion::{criterion_group, criterion_main, Criterion};
use action_suspend::{ActionSuspendManager, SuspendMode, SuspendRequest};

fn bench_suspend(c: &mut Criterion) {
    let manager = ActionSuspendManager::default();
    c.bench_function("mock_suspend", |b| {
        b.iter(|| {
            let _ = manager.execute(SuspendRequest {
                pid: 99999, // fake PID for dry-run
                mode: SuspendMode::Suspend,
                context: Some("bench_run".into()),
            });
        })
    });
}

criterion_group!(benches, bench_suspend);
criterion_main!(benches);