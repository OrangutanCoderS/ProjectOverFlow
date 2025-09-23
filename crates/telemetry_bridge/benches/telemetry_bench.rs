use criterion::{criterion_group, criterion_main, Criterion, black_box};
use serde_json::json;
use std::sync::Arc;

use telemetry_bridge::{TelemetryBridge, BridgeConfig, FileSink, TelemetryPacket};

fn bench_record_file(c: &mut Criterion) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("bench.jsonl");

    let sink = Arc::new(FileSink::new(path).unwrap());
    let bridge = TelemetryBridge::new(BridgeConfig { cache_capacity: 128 });
    bridge.add_sink(sink);

    c.bench_function("record_telemetry_jsonl", |b| {
        b.iter(|| {
            let pkt = TelemetryPacket::new("bench", json!({"x": 42, "y": "abc"}));
            bridge.record(black_box(pkt)).unwrap();
        })
    });
}

criterion_group!(benches, bench_record_file);
criterion_main!(benches);
