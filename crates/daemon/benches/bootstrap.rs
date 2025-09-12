use criterion::{criterion_group, criterion_main, Criterion};
use daemon::Daemon;
use std::fs;
use tempfile::TempDir;

fn config_str() -> &'static str {
    r#"
    [meta]
    schema_version = 1
    phase = "phase1"
    [logging]
    level = "info"
    stdout = true
    rotate_after_mb = 50
    max_days = 7
    [scheduler]
    tick_ms = 100
    watchdog_ms = 500
    [ipc]
    bind = "/tmp/overflow.sock"
    auth_token = "abcdefgh"
    [plugins]
    auto_load = true
    directories = ["plugins"]
    secure_mode_threshold = 7.5
    max_concurrent = 2
    [telemetry]
    enabled = true
    compress = true
    [paths]
    log_dir = "logs"
    export_dir = "logs/snapshots"
    "#
}

fn bench_daemon_bootstrap(c: &mut Criterion) {
    c.bench_function("daemon_new_from_file", |b| {
        b.iter(|| {
            let td = TempDir::new().unwrap();
            let p = td.path().join("cfg.toml");
            fs::write(&p, config_str()).unwrap();
            let d = Daemon::new_from_file(&p).unwrap();
            // start/stop quickly (noop subsystems)
            d.run().unwrap();
            d.shutdown().unwrap();
        });
    });
}

criterion_group!(benches, bench_daemon_bootstrap);
criterion_main!(benches);