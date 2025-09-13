use config::load_from_str;
use criterion::{criterion_group, criterion_main, Criterion};

fn bench_config_load(c: &mut Criterion) {
    let cfg = r#"
    [meta]
    schema_version = 1
    phase = "phase1"

    [logging]
    level = "info"
    stdout = true
    rotate_after_mb = 100
    max_days = 7

    [scheduler]
    tick_ms = 100
    watchdog_ms = 500

    [ipc]
    bind = "/tmp/overflow.sock"
    auth_token = "abcdefgh"

    [plugins]
    auto_load = true
    directories = ["plugins","plugins/rust"]
    secure_mode_threshold = 7.5
    max_concurrent = 2

    [telemetry]
    enabled = true
    compress = true

    [paths]
    log_dir = "logs"
    export_dir = "logs/snapshots"
    "#;

    c.bench_function("config_load_from_str", |b| {
        b.iter(|| {
            let v = load_from_str(cfg).unwrap();
            criterion::black_box(v);
        })
    });
}

criterion_group!(benches, bench_config_load);
criterion_main!(benches);
