use plugin_api::{emit_alert_json, emit_alert_text, log_event, read_config, set_config, ApiConfig};
use pretty_assertions::assert_eq;
use serde_json::json;
use std::{fs, path::PathBuf, time::Duration};

#[test]
fn api_smoke() {
    let tmp = temp_log();
    set_config(ApiConfig {
        log_path: Some(tmp.clone()),
        ..Default::default()
    });

    assert!(log_event("minimal", "init").is_ok());
    assert!(emit_alert_text("minimal", "startup", "ok").is_ok());
    assert!(emit_alert_json("minimal", "payload", &json!({"k":"v"})).is_ok());

    let s = fs::read_to_string(tmp).unwrap();
    assert!(s.lines().count() >= 3);
}

#[test]
fn quota_limits_hold() {
    let tmp = temp_log();
    set_config(ApiConfig {
        window: Duration::from_secs(300),
        max_alerts_per_window: 1,
        max_logs_per_window: 1,
        log_path: Some(tmp),
    });

    log_event("q", "a").unwrap();
    let e = log_event("q", "b").unwrap_err();
    assert!(format!("{e}").contains("quota exceeded"));

    emit_alert_text("q", "r", "m").unwrap();
    let e = emit_alert_text("q", "r", "m2").unwrap_err();
    assert!(format!("{e}").contains("quota exceeded"));
}

// Helper
fn temp_log() -> PathBuf {
    let mut p = std::env::temp_dir();
    p.push(format!("plugin_api_integ_{}", chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)));
    p
}
