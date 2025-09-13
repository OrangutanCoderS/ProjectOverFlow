use clipboard::{calculate_entropy, ClipboardConfig, poll_clipboard, ClipboardEventInfo};

#[test]
fn entropy_sanity() {
    assert!((calculate_entropy(b"aaaaaa") - 0.0).abs() < 1e-9);
    assert!(calculate_entropy(b"abc123!@#") > 3.0);
    let randomish = b"3J9aPqkLx8ZsT1nV0yB4mQfW";
    assert!(calculate_entropy(randomish) > 3.5);
}

#[test]
fn poll_returns_none_when_unchanged() {
    // With empty clipboard (fallback/missing tools), first poll may produce Some(event) with empty data or None.
    // We call twice and ensure second call with same last_hash returns None.
    let cfg = ClipboardConfig::default();
    let first = poll_clipboard(&cfg, "").unwrap_or(None);
    let last_hash = first.as_ref().map(|e| e.content_hash.clone()).unwrap_or_default();
    let second = poll_clipboard(&cfg, &last_hash).unwrap_or(None);
    assert!(second.is_none());
}

#[test]
fn event_validation_minimal() {
    // Construct a minimal synthetic event just to test validate()
    let e = ClipboardEventInfo {
        timestamp: "2025-01-01T00:00:00Z".into(),
        event: "clipboard_update".into(),
        content_hash: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".into(),
        entropy_score: 0.0,
        data_type: "text".into(),
        length: 1,
        source: "unknown".into(),
    };
    assert!(e.validate().is_ok());
}
