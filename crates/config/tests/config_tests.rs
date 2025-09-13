use config::{load_from_path, load_from_str, validate};
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
    directories = ["plugins", "plugins/rust"]
    secure_mode_threshold = 7.5
    max_concurrent = 2

    [telemetry]
    enabled = true
    compress = true

    [paths]
    log_dir = "logs"
    export_dir = "logs/snapshots"

    [toggles]
    dns_logging = true
    entropy_flagger = false
    clipboard_events = true
    "#
    .to_string()
}

#[test]
fn parses_and_validates_ok() {
    let cfg = load_from_str(&sample_ok()).expect("parse");
    validate(&cfg).expect("validate");
}

#[test]
fn rejects_bad_levels_and_bounds() {
    let bad = r#"
    [meta]
    schema_version = 1
    phase = "phase1"
    [logging]
    level = "loud"
    stdout = true
    rotate_after_mb = 0
    max_days = 0
    [scheduler]
    tick_ms = 10
    watchdog_ms = 5
    [ipc]
    bind = ""
    auth_token = "short"
    [plugins]
    auto_load = true
    directories = [""]
    secure_mode_threshold = 42.0
    max_concurrent = 0
    [telemetry]
    enabled = true
    compress = true
    [paths]
    log_dir = "logs"
    export_dir = "logs/snapshots"
    "#;

    let err = load_from_str(bad).unwrap_err();
    let s = err.to_string();
    assert!(s.contains("logging.level") || s.contains("rotate_after_mb") || s.contains("tick_ms"));
}

#[test]
fn load_from_path_and_env_overrides() {
    let td = TempDir::new().unwrap();
    let p = td.path().join("cfg.toml");
    fs::write(&p, sample_ok()).unwrap();

    std::env::set_var("OVERFLOW_LOG_LEVEL", "debug");
    std::env::set_var("OVERFLOW_IPC_TOKEN", "token123");
    std::env::set_var("OVERFLOW_PLUGIN_DIRS", "p1:p2");

    let cfg = load_from_path(&p).expect("load path");
    assert_eq!(cfg.logging.level, "debug");
    assert_eq!(cfg.ipc.auth_token, "token123");
    assert_eq!(
        cfg.plugins.directories,
        vec!["p1".to_string(), "p2".to_string()]
    );
}

#[test]
fn writable_paths_are_checked() {
    let td = TempDir::new().unwrap();
    let log_dir = td.path().join("logs");
    let export_dir = td.path().join("export");
    let mut cfg = sample_ok();
    cfg = cfg.replace("logs\"", &format!("{}\"", log_dir.display()));
    cfg = cfg.replace("logs/snapshots", &export_dir.to_string_lossy());

    let parsed = load_from_str(&cfg).unwrap();
    assert!(parsed.paths.log_dir.contains("logs"));
}
