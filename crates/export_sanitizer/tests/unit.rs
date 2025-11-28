use export_sanitizer::{ExportSanitizer, Salt, SanitizerRules};
use serde_json::json;

fn make_default_sanitizer() -> ExportSanitizer {
    let rules = SanitizerRules::default();
    let salt = Salt::random();
    ExportSanitizer::new(rules, salt)
}

#[test]
fn sanitize_unix_home_path() {
    let sanitizer = make_default_sanitizer();
    let mut packet = json!({
        "path": "/home/alice/secret.txt"
    });

    let report = sanitizer.sanitize_packet(&mut packet);

    let path = packet["path"].as_str().unwrap();
    assert!(path.starts_with("/redacted/path/user_"));
    assert!(report.total_redactions() >= 2); // username + path
}

#[test]
fn sanitize_windows_home_path() {
    let sanitizer = make_default_sanitizer();
    let mut packet = json!({
        "path": "C:\\Users\\bob\\Desktop\\notes.docx"
    });

    let report = sanitizer.sanitize_packet(&mut packet);

    let path = packet["path"].as_str().unwrap();
    assert!(path.starts_with("C:\\redacted\\user_"));
    assert!(report.total_redactions() >= 2);
}

#[test]
fn sanitize_ip_and_mac() {
    let sanitizer = make_default_sanitizer();
    let mut packet = json!({
        "payload": {
            "ip": "192.168.0.42",
            "mac": "aa:bb:cc:dd:ee:ff",
            "note": "connect from 192.168.0.42 (aa:bb:cc:dd:ee:ff)"
        }
    });

    let report = sanitizer.sanitize_packet(&mut packet);

    let ip = packet["payload"]["ip"].as_str().unwrap();
    let mac = packet["payload"]["mac"].as_str().unwrap();
    let note = packet["payload"]["note"].as_str().unwrap();

    assert!(ip.starts_with("ip_"));
    assert!(mac.starts_with("mac_"));
    assert!(note.contains("ip_"));
    assert!(note.contains("mac_"));
    assert!(report.total_redactions() >= 3);
}

#[test]
fn respects_whitelist_paths() {
    let mut rules = SanitizerRules::default();
    rules.whitelist_paths.push("/safe".to_string());

    let salt = Salt::random();
    let sanitizer = ExportSanitizer::new(rules, salt);

    let mut packet = serde_json::json!({
        "path": "/safe/keep.txt"
    });

    let report = sanitizer.sanitize_packet(&mut packet);

    assert_eq!(packet["path"].as_str().unwrap(), "/safe/keep.txt");
    assert_eq!(report.total_redactions(), 0);
}

#[test]
fn trusted_packet_skipped() {
    let sanitizer = make_default_sanitizer();
    let mut packet = json!({
        "meta": { "trusted": true },
        "path": "/home/charlie/secret.txt",
        "payload": {
            "ip": "10.0.0.1"
        }
    });

    let original = packet.clone();
    let report = sanitizer.sanitize_packet(&mut packet);

    assert_eq!(packet, original);
    assert!(report.trusted_skipped);
    assert_eq!(report.total_redactions(), 0);
}

#[test]
fn paranoid_mode_hashes_strings() {
    let mut rules = SanitizerRules::default();
    rules.paranoid = true;

    let salt = Salt::random();
    let sanitizer = ExportSanitizer::new(rules, salt);

    let mut packet = json!({
        "payload": {
            "note": "user logged in from 172.16.0.1",
            "info": "SOME_INTERNAL_MARKER"
        }
    });

    let report = sanitizer.sanitize_packet(&mut packet);

    let note = packet["payload"]["note"].as_str().unwrap();
    let info = packet["payload"]["info"].as_str().unwrap();

    assert!(note.len() >= 64); // hashed
    assert!(info.len() >= 64); // hashed
    assert!(report.total_redactions() >= 2);
}
