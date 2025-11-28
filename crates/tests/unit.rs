use chrono::{TimeZone, Utc};
use memory_replay_engine::{
    engine::DecisionEngine,
    model::{DecisionOutcome, ReplayEvent, ReplayEventKind, ScenarioConfig},
    engine::{MemoryReplayEngine, ScenarioRunner},
};
use serde_json::json;
use std::sync::Arc;

// --- Test helpers ---------------------------------------------------------

fn make_event(seq: u64, baseline_action: &str) -> ReplayEvent {
    ReplayEvent {
        timeline_id: "t1".to_string(),
        sequence_no: seq,
        timestamp: Utc.timestamp_opt(seq as i64, 0).unwrap(),
        origin: "node-A".to_string(),
        kind: ReplayEventKind::Decision,
        payload: json!({ "baseline_action": baseline_action }),
    }
}

#[derive(Clone)]
struct EchoEngine {
    action_prefix: String,
}

impl DecisionEngine for EchoEngine {
    fn evaluate(&self, event: &ReplayEvent) -> DecisionOutcome {
        let action = format!("{}-{}", self.action_prefix, event.sequence_no);
        DecisionOutcome {
            decision_id: format!("dec-{}", event.sequence_no),
            node_id: Some("node-A".to_string()),
            plugin_id: Some("plugin-X".to_string()),
            action,
            metadata: json!({ "kind": format!("{:?}", event.kind) }),
        }
    }
}

// --- Tests ----------------------------------------------------------------

#[test]
fn events_are_sorted_by_timeline_sequence_and_time() {
    let e3 = make_event(3, "allow");
    let e1 = make_event(1, "allow");
    let e2 = make_event(2, "allow");

    let engine = MemoryReplayEngine::new(vec![e3, e1, e2]).expect("non-empty");
    let seqs: Vec<u64> = engine.events().iter().map(|e| e.sequence_no).collect();

    assert_eq!(seqs, vec![1, 2, 3]);
}

#[test]
fn replay_processes_all_events_and_counts_divergence() {
    let e1 = make_event(1, "baseline-1");
    let e2 = make_event(2, "baseline-2");
    let e3 = make_event(3, "baseline-2"); // will diverge

    let engine = MemoryReplayEngine::new(vec![e3, e2, e1]).expect("non-empty");

    let runner = ScenarioRunner {
        id: "scenario-1".to_string(),
        config: ScenarioConfig {
            label: "test-scenario".to_string(),
            description: "basic divergence test".to_string(),
            max_events: None,
        },
        engine: Arc::new(EchoEngine {
            action_prefix: "baseline-2".to_string(),
        }),
    };

    let result = engine.run_scenario(&runner);

    assert_eq!(result.metrics.events_processed, 3);
    // baseline_action: e1="baseline-1", e2="baseline-2", e3="baseline-2"
    // engine action:  "baseline-2-*"
    // So e1 diverges, e2/e3 match.
    assert_eq!(result.metrics.divergent_decisions, 1);
}

#[test]
fn max_events_limit_is_respected() {
    let events: Vec<ReplayEvent> = (0..10)
        .map(|i| make_event(i, "baseline"))
        .collect();

    let engine = MemoryReplayEngine::new(events).expect("non-empty");

    let runner = ScenarioRunner {
        id: "limited".to_string(),
        config: ScenarioConfig {
            label: "limit-5".to_string(),
            description: String::new(),
            max_events: Some(5),
        },
        engine: Arc::new(EchoEngine {
            action_prefix: "p".to_string(),
        }),
    };

    let result = engine.run_scenario(&runner);
    assert_eq!(result.metrics.events_processed, 5);
    assert_eq!(result.decisions.len(), 5);
}

#[test]
fn multiple_scenarios_share_timeline_but_have_independent_metrics() {
    let events: Vec<ReplayEvent> = (0..4)
        .map(|i| make_event(i, "baseline"))
        .collect();

    let engine = MemoryReplayEngine::new(events).expect("non-empty");

    let runner_a = ScenarioRunner {
        id: "A".to_string(),
        config: ScenarioConfig {
            label: "A".to_string(),
            description: "first".to_string(),
            max_events: None,
        },
        engine: Arc::new(EchoEngine {
            action_prefix: "A".to_string(),
        }),
    };

    let runner_b = ScenarioRunner {
        id: "B".to_string(),
        config: ScenarioConfig {
            label: "B".to_string(),
            description: "second".to_string(),
            max_events: Some(2),
        },
        engine: Arc::new(EchoEngine {
            action_prefix: "B".to_string(),
        }),
    };

    let results = engine.run_scenarios(&[runner_a, runner_b]);
    assert_eq!(results.len(), 2);

    let r_a = &results[0];
    let r_b = &results[1];

    assert_eq!(r_a.metrics.events_processed, 4);
    assert_eq!(r_b.metrics.events_processed, 2);
}
