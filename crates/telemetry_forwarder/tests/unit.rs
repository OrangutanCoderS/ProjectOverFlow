use telemetry_forwarder::{
    model::{TelemetryCategory, TelemetryPayload},
    sink::InMemorySink,
    TelemetryForwarder,
};
use serde_json::json;
use std::sync::Arc;

#[tokio::test]
async fn test_immediate_send_builds_envelope() {
    let sink = Arc::new(InMemorySink::new());
    let forwarder = TelemetryForwarder::new("node-1".to_string(), Arc::clone(&sink), 16);

    let payload = TelemetryPayload {
        category: TelemetryCategory::Cpu,
        body: json!({"usage": 0.91}),
    };

    let result = forwarder
        .send_telemetry_now(payload, Some("trace-123".to_string()), None)
        .await
        .expect("send should succeed");

    assert!(result.accepted);
    let envs = sink.envelopes();
    assert_eq!(envs.len(), 1);
    let env = &envs[0];

    assert_eq!(env.origin_id, "node-1");
    assert_eq!(env.category, TelemetryCategory::Cpu);
    assert!(env.timestamp <= chrono::Utc::now());
    assert_eq!(env.trace_id.as_deref(), Some("trace-123"));
}

#[tokio::test]
async fn test_queue_respects_capacity() {
    let sink = Arc::new(InMemorySink::new());
    // Tiny queue capacity so we can hit QueueFull quickly.
    let forwarder = TelemetryForwarder::new("node-1".to_string(), Arc::clone(&sink), 1);

    let payload = TelemetryPayload {
        category: TelemetryCategory::Memory,
        body: json!({"used": 1024}),
    };

    // First enqueue should succeed.
    let r1 = forwarder.queue_telemetry(payload.clone()).await;
    assert!(r1.is_ok());

    // Second enqueue should hit QueueFull (because worker might not have drained yet).
    let r2 = forwarder.queue_telemetry(payload).await;
    assert!(r2.is_err());
    let err = r2.unwrap_err().to_string();
    assert!(
        err.contains("queue is full"),
        "expected QueueFull, got: {err}"
    );
}

#[tokio::test]
async fn test_heartbeat_helper() {
    let sink = Arc::new(InMemorySink::new());
    let forwarder = TelemetryForwarder::new("node-xyz".to_string(), Arc::clone(&sink), 8);

    let body = json!({"status": "ok"});
    let res = forwarder.queue_heartbeat(body).await.expect("enqueue ok");
    assert!(res.accepted);

    // Give the worker a tick.
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;

    let envs = sink.envelopes();
    assert_eq!(envs.len(), 1);
    assert_eq!(envs[0].category, TelemetryCategory::Heartbeat);
    assert_eq!(envs[0].origin_id, "node-xyz");
}
