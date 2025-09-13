use criterion::{criterion_group, criterion_main, Criterion};
use sandbox::{SandboxConfig, run_in_sandbox};

fn bench_sandbox_run(c: &mut Criterion) {
    // Use config with default profile path (points to crates/sandbox/profiles/minimal.sb)
    let mut cfg = SandboxConfig::default();

    // Override profile path explicitly for clarity
    cfg.profile_path = std::path::PathBuf::from(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/profiles/minimal.sb"
    ));

    c.bench_function("sandbox_run_echo", |b| {
        b.iter(|| {
            run_in_sandbox(&cfg, "echo", &["hello"])
                .expect("sandbox run failed")
        })
    });
}

criterion_group!(benches, bench_sandbox_run);
criterion_main!(benches);