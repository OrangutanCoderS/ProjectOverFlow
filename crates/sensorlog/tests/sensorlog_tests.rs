use sensorlog::{SensorConfig, poll_sensors, SensorAccessInfo};
use pretty_assertions::assert_eq;

#[test]
fn validate_model_minimal() {
    let ev = SensorAccessInfo {
        timestamp: "2025-01-01T00:00:00Z".into(),
        sensor: "camera".into(),
        access_type: "read".into(),
        pid: None,
        process: Some("com.apple.FaceTime".into()),
        last_used_unix: Some(1_726_000_000),
        foreground: None,
    };
    assert!(ev.validate().is_ok());
}

#[test]
fn poll_returns_vector_or_empty_not_error() {
    // On non-macOS this will return empty vector; it must not error or panic.
    let cfg = SensorConfig::default();
    let r = poll_sensors(&cfg).expect("poll_sensors should not error");
    assert!(r.len() <= cfg.max_entries_per_sample);
}
