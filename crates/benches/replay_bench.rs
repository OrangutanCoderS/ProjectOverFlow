use chrono::{TimeZone, Utc};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use memory_replay_engine::{
    engine::{DecisionEngine, MemoryReplayEngine, ScenarioRunner},
    model::{DecisionOutcome, ReplayEvent, ReplayEventKind, ScenarioConfig},
};
use rand::{rngs::StdRng, Rng, SeedableRng};
use serde_json::json;
use std::sync::Arc;

// Simple deterministic engine for benchmarking.
struct BenchEngine;

impl DecisionEngine for BenchEngine {
    fn evaluate(&self, event: &ReplayEvent) -> DecisionOutcome {
        // Minimal work: one string concat + tiny JSON.
        let action = if event.sequence_no % 2 == 0 {
            "even-allow".to_string()
        } else {
            "odd-allow".to_string()
        };

        DecisionOutcome {
            decision_id: format!("bench-{}", event.sequence_no),
            node_id: None,
            plugin_id: None,
            action,
            metadata: json!({
                "kind": format!("{:?}", event.kind),
            }),
        }
    }
}

fn make_timeline(n: usize) -> Vec<ReplayEvent> {
    let mut rng = StdRng::seed_from_u64(42);

    (0..n)
        .map(|i| {
            let baseline = if i % 3 == 0 { "even-allow" } else { "odd-allow" };
            ReplayEvent {
                timeline_id: "bench-t1".to_string(),
                sequence_no: i as u64,
                timestamp: Utc.timestamp_opt(i as i64, 0).unwrap(),
                origin: "bench-node".to_string(),
                kind: ReplayEventKind::Decision,
                payload: json!({
                    "baseline_action": baseline,
                    "score": rng.gen::<f64>(),
                }),
            }
        })
        .collect()
}

fn bench_memory_replay(c: &mut Criterion) {
    let events = make_timeline(10_000);
    let engine = MemoryReplayEngine::new(events).expect("non-empty");

    let runner = ScenarioRunner {
        id: "bench-scenario".to_string(),
        config: ScenarioConfig {
            label: "bench".to_string(),
            description: "10k events replay".to_string(),
            max_events: None,
        },
        engine: Arc::new(BenchEngine),
    };

    c.bench_function("memory_replay_10k_events", |b| {
        b.iter(|| {
            let result = engine.run_scenario(&runner);
            black_box(result.metrics.events_processed);
        });
    });
}

criterion_group!(benches, bench_memory_replay);
criterion_main!(benches);
