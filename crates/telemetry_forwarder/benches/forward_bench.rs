use criterion::{black_box, criterion_group, criterion_main, Criterion};
use serde_json::json;
use std::sync::Arc;
use tokio::runtime::Runtime;

use telemetry_forwarder::{
    model::{TelemetryCategory, TelemetryPayload},
    sink::InMemorySink,
    TelemetryForwarder,
};

fn bench_forwarder(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("telemetry_forwarder_queue_telemetry", |b| {
        b.to_async(&rt).iter(|| async {
            let sink = Arc::new(InMemorySink::new());
            let forwarder =
                TelemetryForwarder::new("bench-node".to_string(), Arc::clone(&sink), 1024);

            let payload = TelemetryPayload {
                category: TelemetryCategory::Cpu,
                body: json!({"usage": 0.73, "core_count": 10}),
            };

            forwarder.queue_telemetry(black_box(payload)).await.unwrap();
        });
    });
}

criterion_group!(benches, bench_forwarder);
criterion_main!(benches);