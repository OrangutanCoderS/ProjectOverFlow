use criterion::{black_box, criterion_group, criterion_main, Criterion};
use overflow_utils::*;
use std::time::Duration;

fn bench_utils(c: &mut Criterion) {
    c.bench_function("fmt_bytes_iec", |b| {
        b.iter(|| black_box(fmt_bytes_iec(black_box(123_456_789))))
    });

    let data: Vec<u8> = (0..65536).map(|i| (i % 251) as u8).collect();
    c.bench_function("shannon_entropy 64KiB", |b| {
        b.iter(|| black_box(shannon_entropy(black_box(&data))))
    });

    c.bench_function("subprocess echo", |b| {
        b.iter(|| {
            let _ = run_command_with_timeout("echo", ["hi"], Duration::from_millis(50)).unwrap();
        })
    });
}

criterion_group!(benches, bench_utils);
criterion_main!(benches);
