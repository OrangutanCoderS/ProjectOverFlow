use std::time::Duration;

use chrono::{TimeZone, Utc};
use criterion::{criterion_group, criterion_main, Criterion};
use telemetry_pattern_miner::{PatternMiner, PatternStore, TelemetryEvent};

fn make_event(ts: i64, plugin_idx: usize) -> TelemetryEvent {
    TelemetryEvent {
        timestamp: Utc.timestamp_opt(ts, 0).single().unwrap(),
        session_id: Some("bench".to_string()),
        process_id: Some(42),
        origin_plugin: Some("origin".to_string()),
        plugin: format!("plugin_{}", plugin_idx),
        result: Some("ok".to_string()),
        entropy: Some(7.0 + (plugin_idx as f64 * 0.1)),
        failed: false,
    }
}

fn bench_miner(c: &mut Criterion) {
    let store = PatternStore::open_in_memory().expect("store");
    let miner = PatternMiner::with_params(store, 4, 3);

    // Build a synthetic event stream.
    let mut events = Vec::new();
    for i in 0..1_000 {
        let plugin_idx = i % 10;
        events.push(make_event(i as i64, plugin_idx));
    }

    c.bench_function("telemetry_pattern_miner_mine_batch", |b| {
        b.iter(|| {
            miner.mine_batch(&events).unwrap();
        })
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .warm_up_time(Duration::from_secs(2))
        .measurement_time(Duration::from_secs(5));
    targets = bench_miner
}
criterion_main!(benches);