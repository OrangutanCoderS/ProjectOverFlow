use criterion::{criterion_group, criterion_main, Criterion};
use trigger_thresholds::{evaluate, load_thresholds};
use tempfile::NamedTempFile;

fn bench_eval(c: &mut Criterion) {
    let tmp = NamedTempFile::new().unwrap();
    std::fs::write(
        tmp.path(),
        r#"{
            "entropy": {"warn": 6.5, "block": 7.5},
            "cpu": {"warn": 80, "block": 95},
            "net_spike_kb": {"warn": 400, "block": 700},
            "adaptive": false
        }"#,
    ).unwrap();

    load_thresholds(tmp.path().to_str().unwrap()).unwrap();
    c.bench_function("threshold_evaluate_entropy", |b| {
        b.iter(|| evaluate("entropy", 7.2).unwrap());
    });
}

criterion_group!(benches, bench_eval);
criterion_main!(benches);