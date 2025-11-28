use criterion::{criterion_group, criterion_main, Criterion};
use process_spoofing::*;

fn bench_spoof_query(c: &mut Criterion) {
    let mgr = SpoofManager::new();
    let s = SpoofIdentity {
        real_pid: 42,
        fake_pid: 420,
        fake_ppid: 1,
        fake_name: None,
        fake_env: None,
        created_at: chrono::Utc::now(),
        ttl: None,
    };
    mgr.add_spoof(s).unwrap();
    c.bench_function("spoof_query", |b| b.iter(|| mgr.query(42)));
}

criterion_group!(benches, bench_spoof_query);
criterion_main!(benches);