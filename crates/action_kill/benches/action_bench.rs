use criterion::{criterion_group, criterion_main, Criterion};
use std::sync::Arc;
use action_kill::*;
use tempfile::tempdir;

fn bench_softkill(c: &mut Criterion) {
    let dir = tempdir().unwrap();
    let log_path = dir.path().join("bench.jsonl");
    let logger = Arc::new(AuditLogger::new(log_path.to_str().unwrap()).unwrap());
    let ctrl = Arc::new(controller::mock::MockProcController::new());
    ctrl.insert(42);
    let mgr = ActionKillManager::new(ctrl, logger);

    c.bench_function("mock_softkill", |b| {
        b.iter(|| {
            let req = KillRequest::new(42, KillMode::SoftKill, "bench", false);
            let _ = mgr.execute(&req);
        })
    });
}

criterion_group!(benches, bench_softkill);
criterion_main!(benches);