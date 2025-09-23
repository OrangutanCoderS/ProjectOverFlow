use criterion::{criterion_group, criterion_main, Criterion};
use plugin_engine::{load_plugin, run_plugin};
use std::path::PathBuf;

fn bench_plugin_engine(c: &mut Criterion) {
    let path = PathBuf::from("plugins/rust/minimal_plugin/target/release/libminimal_plugin.dylib");

    c.bench_function("load_and_run_minimal_plugin", |b| {
        b.iter(|| {
            let meta = load_plugin(path.clone()).expect("load failed");
            let _ = run_plugin(&meta.name, "hello").expect("run failed");
        })
    });
}

criterion_group!(benches, bench_plugin_engine);
criterion_main!(benches);