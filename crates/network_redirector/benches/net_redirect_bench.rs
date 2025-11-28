use criterion::{criterion_group, criterion_main, Criterion};
use std::sync::Arc;
use network_redirector::*;

fn bench_submit(c: &mut Criterion) {
    let mut group = c.benchmark_group("redirector");

    let mgr = NetworkRedirectManager::new(
        Arc::new(MockBackend::new()),
        ManagerConfig::default(),
        "logs/bench_submit.json",
    );

    let req = RedirectRequest {
        mode: RedirectMode::Block,
        target_pid: None,
        match_type: MatchType::Domain,
        match_value: "test.com".into(),
        redirect_to: None,
        ttl_secs: None,
        reason: None,
        plugin: None,
        metadata: None,
        dry_run: false,
        created_by: None,
    };

    group.bench_function("submit_block", |b| {
        b.iter(|| {
            let _ = mgr.submit(req.clone());
        })
    });

    group.finish();
}

fn bench_submit_and_remove(c: &mut Criterion) {
    let mut group = c.benchmark_group("redirector");

    let mgr = NetworkRedirectManager::new(
        Arc::new(MockBackend::new()),
        ManagerConfig::default(),
        "logs/bench_submit_remove.json",
    );

    let req = RedirectRequest {
        mode: RedirectMode::Block,
        target_pid: None,
        match_type: MatchType::Domain,
        match_value: "remove.com".into(),
        redirect_to: None,
        ttl_secs: None,
        reason: None,
        plugin: None,
        metadata: None,
        dry_run: false,
        created_by: None,
    };

    group.bench_function("submit_and_remove", |b| {
        b.iter(|| {
            let rule = mgr.submit(req.clone()).unwrap();
            let _ = mgr.remove(&rule.id);
        })
    });

    group.finish();
}

criterion_group!(benches, bench_submit, bench_submit_and_remove);
criterion_main!(benches);