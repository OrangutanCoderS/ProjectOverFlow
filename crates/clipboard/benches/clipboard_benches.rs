use clipboard::{calculate_entropy};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_entropy_small(c: &mut Criterion) {
    let data = vec![b'a'; 1024];
    c.bench_function("entropy_1KB", |b| {
        b.iter(|| black_box(calculate_entropy(&data)))
    });
}

fn bench_entropy_large(c: &mut Criterion) {
    let data = vec![42u8; 1024 * 1024];
    c.bench_function("entropy_1MB", |b| {
        b.iter(|| black_box(calculate_entropy(&data)))
    });
}

criterion_group!(benches, bench_entropy_small, bench_entropy_large);
criterion_main!(benches);
