use chrono::{TimeZone, Utc};
use telemetry_pattern_miner::{PatternMiner, PatternStore, TelemetryEvent};

fn make_event(ts: i64, plugin: &str) -> TelemetryEvent {
    TelemetryEvent {
        timestamp: Utc.timestamp_opt(ts, 0).single().unwrap(),
        session_id: Some("sess-1".to_string()),
        process_id: Some(123),
        origin_plugin: Some("origin".to_string()),
        plugin: plugin.to_string(),
        result: Some("ok".to_string()),
        entropy: Some(7.5),
        failed: false,
    }
}

#[test]
fn basic_pattern_mining_works() {
    let store = PatternStore::open_in_memory().expect("store");
    let miner = PatternMiner::with_params(store, 3, 2);

    // Sequence: A -> B -> C -> A -> B
    let events = vec![
        make_event(1, "A"),
        make_event(2, "B"),
        make_event(3, "C"),
        make_event(4, "A"),
        make_event(5, "B"),
    ];

    miner.mine_batch(&events).expect("mine_batch");

    let top = miner.store().top_patterns(10).expect("top");
    assert!(!top.is_empty(), "expected at least one mined pattern");

    // ensure we have some chain of length >= 2
    assert!(top.iter().any(|p| p.length >= 2));
}