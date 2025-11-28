use criterion::{criterion_group, criterion_main, Criterion};
use chrono::Utc;
use pattern_cache::{PatternCacheEngine, model::NewPattern};

fn bench_insert(c: &mut Criterion) {
    let path = "bench_cache.db";
    let _ = std::fs::remove_file(path);

    let cache = PatternCacheEngine::new(path).unwrap();

    c.bench_function("pattern_cache_insert", |b| {
        b.iter(|| {
            let p = NewPattern {
                category: "cpu".to_string(),
                fingerprint: "fp-bench".to_string(),
                score: 0.77,
                first_seen: Utc::now(),
                last_seen: Utc::now(),
            };
            let _ = cache.insert(&p);
        });
    });
}

criterion_group!(benches, bench_insert);
criterion_main!(benches);
