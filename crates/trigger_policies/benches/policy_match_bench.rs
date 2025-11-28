use criterion::{black_box, criterion_group, criterion_main, Criterion};
use trigger_policies::{init_policy_cache, find_match};
use std::collections::HashMap;
use std::fs;
use tempfile::NamedTempFile;
use std::io::Write;

fn bench_policy_eval(c: &mut Criterion) {
    // Create a temp YAML for benchmarking to avoid file dependency
    let mut tmp = NamedTempFile::new().unwrap();
    write!(
        tmp,
        "policies:
          - id: bench1
            trigger_name: high_cpu
            scope: process
            conditions: {{ cpu: \">80\" }}
            actions: [\"throttle\"]
            priority: 10
            cooldown: 5
          - id: bench2
            trigger_name: high_mem
            scope: process
            conditions: {{ mem: \">2048\" }}
            actions: [\"suspend\"]
            priority: 9
            cooldown: 10
        "
    )
    .unwrap();

    init_policy_cache(tmp.path().to_str().unwrap()).unwrap();

    // Build a test context
    let mut ctx = HashMap::new();
    ctx.insert("cpu".to_string(), "85".to_string());
    ctx.insert("mem".to_string(), "2300".to_string());

    // Benchmark the matcher
    c.bench_function("policy_match_eval", |b| {
        b.iter(|| {
            let _ = find_match(black_box(&ctx)).unwrap();
        });
    });
}

criterion_group!(benches, bench_policy_eval);
criterion_main!(benches);