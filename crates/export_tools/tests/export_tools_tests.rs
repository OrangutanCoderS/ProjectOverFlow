use export_tools::formats::{csv, json, snapshot::Snapshot};
use export_tools::filters::redact::redact_json_in_place;
use export_tools::ExportError;
use pretty_assertions::assert_eq;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;
use chrono::{Utc, DateTime};

#[derive(Debug, Serialize, Deserialize)]
struct Row {
    id: u32,
    user: String,
    password: String,
    note: String,
    when: DateTime<Utc>,
}

fn sample_rows() -> Vec<Row> {
    vec![
        Row { id: 1, user: "a".into(), password: "sekret".into(), note: "hello".into(), when: Utc::now() },
        Row { id: 2, user: "b".into(), password: "hunter2".into(), note: "world".into(), when: Utc::now() },
    ]
}

#[test]
fn json_export_with_redaction() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("out.json");

    let cfg = json::JsonConfig {
        pretty: true,
        redact_keys: vec!["password".into()],
    };

    json::export_json(&path, &sample_rows(), &cfg).unwrap();
    let body = fs::read_to_string(path).unwrap();
    assert!(body.contains(r#""password": "[REDACTED]""#));
    assert!(body.contains(r#""user": "a""#));
}

#[test]
fn csv_export_with_header_and_redaction() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("out.csv");

    let cfg = csv::CsvConfig {
        header: true,
        redact_keys: vec!["password".into()],
    };

    csv::export_csv(&path, &sample_rows(), &cfg).unwrap();
    let body = fs::read_to_string(path).unwrap();
    // header should contain all keys (order stable but not guaranteed specific here)
    assert!(body.lines().next().unwrap().contains("user"));
    assert!(body.contains("[REDACTED]"));
}

#[test]
fn csv_export_rejects_non_object_rows() {
    #[derive(Serialize)]
    struct NonObj(u32);
    let rows = vec![NonObj(5)];

    let dir = tempdir().unwrap();
    let path = dir.path().join("bad.csv");
    let cfg = csv::CsvConfig::default();

    let e = csv::export_csv(&path, &rows, &cfg).unwrap_err();
    match e {
        ExportError::Invalid(msg) => assert!(msg.contains("CSV row must serialize to a JSON object")),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn snapshot_accumulates_bounds() {
    let mut snap = Snapshot::default();
    let t1 = Utc::now();
    let t0 = t1 - chrono::Duration::hours(1);
    let t2 = t1 + chrono::Duration::hours(1);

    snap.update(Some(t1));
    snap.update(Some(t0));
    snap.update(Some(t2));
    snap.update(None); // still increments total

    assert_eq!(snap.total, 4);
    assert_eq!(snap.first, Some(t0));
    assert_eq!(snap.last,  Some(t2));
}

#[test]
fn redact_recursive_in_place() {
    let mut v: Value = json!({
        "password": "secret",
        "nested": { "password": "x", "keep": 1 },
        "list": [ {"password":"y"}, {"ok":true} ]
    });
    redact_json_in_place(&mut v, &vec!["password".into()]);
    let s = v.to_string();
    assert!(s.contains(r#""password":"[REDACTED]""#));
    assert!(s.contains(r#""keep":1"#));
}
