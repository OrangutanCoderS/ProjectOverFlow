use std::collections::HashMap;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use action_inject_env::{ActionInjectEnvManager, EnvInjectionMode, InjectRequest};

fn bench_mock_injection(c: &mut Criterion) {
    // We benchmark manager validation + light map cloning in DryRun mode,
    // which is the hot path before actual spawning.
    let mgr = ActionInjectEnvManager::default();

    let mut env = HashMap::new();
    env.insert("A".into(), "1".into());
    env.insert("B".into(), "2".into());
    env.insert("C".into(), "3".into());

    let req = InjectRequest {
        program: "/usr/bin/env".into(),
        args: vec![],
        env,
        inherit: true,
        mode: EnvInjectionMode::DryRun,
        context: Some("bench".into()),
    };

    c.bench_function("inject_env_dryrun", |b| {
        b.iter(|| {
            let _ = mgr.execute(black_box(req.clone()));
        })
    });
}

criterion_group!(benches, bench_mock_injection);
criterion_main!(benches);