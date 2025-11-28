use trigger_thresholds::{evaluate, load_thresholds};

#[test]
fn test_threshold_eval() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(
        tmp.path(),
        r#"{
            "entropy": {"warn": 6.5, "block": 7.5},
            "cpu": {"warn": 80, "block": 95},
            "net_spike_kb": {"warn": 400, "block": 700},
            "adaptive": false
        }"#,
    ).unwrap();

    load_thresholds(tmp.path().to_str().unwrap()).unwrap();
    assert_eq!(evaluate("entropy", 5.0).unwrap(), "OK");
    assert_eq!(evaluate("entropy", 6.6).unwrap(), "WARN");
    assert_eq!(evaluate("entropy", 8.0).unwrap(), "BLOCK");
}