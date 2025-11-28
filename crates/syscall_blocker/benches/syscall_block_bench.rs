use criterion::{criterion_group, criterion_main, Criterion};
use syscall_blocker::{handle_request, SysBlockRequest};

fn bench_syscall_block(c: &mut Criterion) {
    c.bench_function("mock_syscall_block", |b| {
        b.iter(|| {
            let req = SysBlockRequest {
                pid: 1000,
                syscall: "unlink".into(),
                mode: "log".into(),
            };
            let _ = handle_request(req);
        });
    });
}

criterion_group!(benches, bench_syscall_block);
criterion_main!(benches);