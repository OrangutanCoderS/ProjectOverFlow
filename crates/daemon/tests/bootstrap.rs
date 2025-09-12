use daemon::Daemon;
use std::fs;
use tempfile::TempDir;

fn sample_ok() -> String {
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
    .to_string()
}

#[test]
fn daemon_bootstrap_smoke() {
    let td = TempDir::new().unwrap();
    let cfgp = td.path().join("cfg.toml");
    fs::write(&cfgp, sample_ok()).unwrap();

    let d = Daemon::new_from_file(&cfgp).expect("daemon create");
    d.run().expect("daemon run");
    d.shutdown().expect("daemon stop");
}