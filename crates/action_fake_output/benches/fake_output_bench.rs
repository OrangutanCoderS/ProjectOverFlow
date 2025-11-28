use action_fake_output::controller::PlatformFakeOutput;
use action_fake_output::FakeOutputController;
use criterion::{criterion_group, criterion_main, Criterion};

fn bench_stdout(c: &mut Criterion) {
    c.bench_function("inject_stdout_fake", |b| {
        b.iter(|| {
            let pid = std::process::id() as i32;
            let _ = PlatformFakeOutput::inject_stdout(pid, "Benchmarking fake stdout...");
        })
    });
}

criterion_group!(benches, bench_stdout);
criterion_main!(benches);