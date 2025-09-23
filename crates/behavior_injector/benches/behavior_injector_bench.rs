//! Benchmark suite for behavior_injector
//! Uses Criterion.rs for stable benchmarking.

use criterion::{criterion_group, criterion_main, Criterion};
use behavior_injector::{
    dispatch_via_global, InterventionRequest, install_global_injector,
    InjectorConfig, BehaviorInjector, Action,
};

fn bench_dispatch(c: &mut Criterion) {
    // Initialize global injector once
    let config = InjectorConfig { dry_run: true, ..Default::default() };
    let injector = BehaviorInjector::new_default(config).unwrap();
    install_global_injector(injector);

    // Build a request with the new constructor
    let req = InterventionRequest::new(Action::ActionFakeOutput, Some(12345), None);

    c.bench_function("dispatch_via_global", |b| {
        b.iter(|| {
            let _ = dispatch_via_global(req.clone());
        })
    });
}

criterion_group!(benches, bench_dispatch);
criterion_main!(benches);
