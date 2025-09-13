use overflow_utils::{bytes_human, file_entropy_limited, run_cmd, unix_time_ms, utc_iso8601};
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

#[test]
fn bytes_format_is_sane() {
    assert_eq!(bytes_human(0), "0 B");
    assert!(bytes_human(1024).contains("KiB")); // FIXED: expect KiB not KB
    assert!(bytes_human(1024 * 1024).contains("MiB"));
}

#[test]
fn time_helpers_work() {
    let now = unix_time_ms();
    assert!(now > 0);
    let iso = utc_iso8601();
    assert!(iso.contains("T") && iso.ends_with('Z'));
}

#[test]
fn entropy_bounds() {
    let low = b"aaaaaaaaaaaaaa";
    let high = b"abcdef1234567890";
    let low_h = overflow_utils::shannon_entropy(low);
    let high_h = overflow_utils::shannon_entropy(high);
    assert!(low_h < high_h);
    assert!(low_h >= 0.0 && low_h <= 8.0);
    assert!(high_h >= 0.0 && high_h <= 8.0);
}

#[test]
fn file_entropy_capped_and_atomic_json() {
    // create a temp file
    let tmp_dir = tempfile::tempdir().unwrap();
    let tmp_path = tmp_dir.path().join("test.txt");
    fs::write(&tmp_path, b"aaaaa").unwrap();

    let e = file_entropy_limited(&tmp_path, 1024).unwrap();
    assert!(e >= 0.0 && e <= 8.0);

    // test atomic json write
    let json_path = tmp_dir.path().join("out.json");
    overflow_utils::write_json_atomic(&json_path, &serde_json::json!({"a":1})).unwrap();
    let content: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&json_path).unwrap()).unwrap();
    assert_eq!(content["a"], 1);
}

#[test]
fn subprocess_timeout_and_exit() {
    // should finish quickly
    let out = run_cmd("echo", &["hi"], Some(1000)).unwrap();
    assert!(out.contains("hi"));

    // should timeout safely
    let res = run_cmd("sleep", &["2"], Some(100)).unwrap();
    assert_eq!(res, "");
}
