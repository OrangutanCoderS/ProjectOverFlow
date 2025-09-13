use securemode::{evaluate_secure_mode, SecureConfig, SecureLevel};

#[test]
fn evaluates_normal() {
    let cfg = SecureConfig::default();
    let ev = evaluate_secure_mode(&cfg, 0.1, 0, "baseline").unwrap();
    assert_eq!(ev.level, SecureLevel::Normal);
}

#[test]
fn evaluates_elevated() {
    let cfg = SecureConfig::default();
    let ev = evaluate_secure_mode(&cfg, 0.4, 1, "moderate suspicion").unwrap();
    assert_eq!(ev.level, SecureLevel::Elevated);
}

#[test]
fn evaluates_contained() {
    let cfg = SecureConfig::default();
    let ev = evaluate_secure_mode(&cfg, 0.7, 3, "high suspicion").unwrap();
    assert_eq!(ev.level, SecureLevel::Contained);
}

#[test]
fn evaluates_locked() {
    let cfg = SecureConfig::default();
    let ev = evaluate_secure_mode(&cfg, 0.9, 12, "critical threat").unwrap();
    assert_eq!(ev.level, SecureLevel::Locked);
}
