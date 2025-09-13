use criterion::{black_box, criterion_group, criterion_main, Criterion};
use overflow_core::*;
use serde_json as json;

fn make_process() -> ProcessInfo {
    ProcessInfo {
        base: BaseEvent::new(2222, "process"),
        name: "ShadowTrace".into(),
        ppid: 1,
        user: "root".into(),
        cpu_pct: 1.2,
        mem_mb: 42.0,
        threads: 2,
        cmdline: vec!["shadowtrace".into(), "--flag".into()],
        start_time_ms: 1_700_000_000_000,
    }
}

fn bench_serde(c: &mut Criterion) {
    // ---- ProcessInfo JSON ----
    c.bench_function("ProcessInfo to_json", |b| {
        b.iter(|| {
            let p = make_process();
            black_box(p.to_json().unwrap())
        })
    });

    c.bench_function("ProcessInfo from_json", |b| {
        let s = make_process().to_json().unwrap();
        b.iter(|| {
            let _: ProcessInfo = json::from_str(black_box(&s)).unwrap();
        })
    });

    // ---- AnyEvent JSON ----
    c.bench_function("AnyEvent serialize", |b| {
        b.iter(|| {
            let e = AnyEvent::ProcessInfo(make_process());
            black_box(json::to_string(&e).unwrap())
        })
    });

    c.bench_function("AnyEvent deserialize", |b| {
        let e = AnyEvent::ProcessInfo(make_process());
        let s = json::to_string(&e).unwrap();
        b.iter(|| {
            let _: AnyEvent = json::from_str(black_box(&s)).unwrap();
        })
    });

    // ---- Validation ----
    c.bench_function("ProcessInfo validate", |b| {
        b.iter(|| {
            let p = make_process();
            black_box(p.validate().unwrap());
        })
    });

    // ---- Content hash ----
    c.bench_function("BaseEvent content_hash_hex", |b| {
        b.iter(|| {
            let p = make_process();
            black_box(p.base.content_hash_hex(&p).unwrap())
        })
    });
}

criterion_group!(benches, bench_serde);
criterion_main!(benches);
