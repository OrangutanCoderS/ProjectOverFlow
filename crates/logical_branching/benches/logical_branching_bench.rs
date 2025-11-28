use criterion::{criterion_group, criterion_main, Criterion};
use hashbrown::{HashMap, HashSet};

use logical_branching::{
    load_from_yaml_str, ActionId, LogicalBranchingEngine, PluginRuntimeState, RuntimeSnapshot,
};

const EXAMPLE_YAML: &str = r#"
rules:
  - context_conditions:
      - high_memory_pressure
    plugin_states:
      fake_net: active
    system_modifiers:
      secure_mode: off
    allowed_responses:
      - action_fake_output
      - action_throttle
    blocked_paths:
      - action_kill
    fallback:
      - action_delay
"#;

fn allowed_action_set() -> HashSet<ActionId> {
    let mut set = HashSet::new();
    for a in &[
        "action_kill",
        "action_fake_output",
        "action_throttle",
        "action_delay",
    ] {
        set.insert((*a).to_string());
    }
    set
}

fn build_snapshot_matching() -> RuntimeSnapshot {
    let mut ctx = HashSet::new();
    ctx.insert("high_memory_pressure".to_string());

    let mut plugin_states = HashMap::new();
    plugin_states.insert("fake_net".to_string(), PluginRuntimeState::Active);

    let mut modifiers = HashMap::new();
    modifiers.insert("secure_mode".to_string(), "off".to_string());

    RuntimeSnapshot {
        context_flags: ctx,
        plugin_states,
        system_modifiers: modifiers,
    }
}

fn bench_load_only(c: &mut Criterion) {
    let allowed = allowed_action_set();

    c.bench_function("logical_branching_load_only", |b| {
        b.iter(|| {
            let _engine: LogicalBranchingEngine =
                load_from_yaml_str(EXAMPLE_YAML, Some(&allowed)).unwrap();
        });
    });
}

fn bench_decide_hot_path(c: &mut Criterion) {
    let allowed = allowed_action_set();
    let engine: LogicalBranchingEngine =
        load_from_yaml_str(EXAMPLE_YAML, Some(&allowed)).expect("valid config");

    let snapshot = build_snapshot_matching();
    let candidates: Vec<ActionId> = vec![
        "action_kill".to_string(),
        "action_fake_output".to_string(),
        "action_throttle".to_string(),
    ];

    c.bench_function("logical_branching_decide_hot", |b| {
        b.iter(|| {
            let decision = engine.decide(&snapshot, &candidates);
            assert_eq!(decision.allowed.len(), 2);
        });
    });
}

criterion_group!(benches, bench_load_only, bench_decide_hot_path);
criterion_main!(benches);