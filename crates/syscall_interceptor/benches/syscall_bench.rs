use criterion::{criterion_group, criterion_main, Criterion};
use syscall_interceptor::{SyscallEvent, SyscallInterceptor};
use chrono::Utc;

fn bench_push_recv(c: &mut Criterion) {
    let mut icp = SyscallInterceptor::new();
    icp.attach().unwrap();

    c.bench_function("push+recv", |b| {
        b.iter(|| {
            let ev = SyscallEvent {
                pid: 42,
                name: "read".into(),
                args: vec!["fd=3".into()],
                timestamp: Utc::now().to_rfc3339(),
            };
            icp.push_mock(ev).unwrap();
            let _ = icp.try_recv().unwrap();
        })
    });
}

criterion_group!(benches, bench_push_recv);
criterion_main!(benches);