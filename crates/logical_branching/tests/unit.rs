use hashbrown::{HashMap, HashSet};

use logical_branching::{
    load_from_yaml_str, ActionId, LogicalBranchError, LogicalBranchingEngine, PluginRuntimeState,
    RuntimeSnapshot,
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

#[test]
fn load_config_without_validation() {
    let engine: LogicalBranchingEngine =
        load_from_yaml_str(EXAMPLE_YAML, None).expect("config should load");

    let snapshot = build_snapshot_matching();
    let candidates: Vec<ActionId> = vec![
        "action_kill".to_string(),
        "action_fake_output".to_string(),
        "action_throttle".to_string(),
    ];

    let decision = engine.decide(&snapshot, &candidates);

    assert_eq!(
        decision.allowed,
        vec![
            "action_fake_output".to_string(),
            "action_throttle".to_string()
        ]
    );
    assert!(decision.blocked.contains(&"action_kill".to_string()));
    assert!(decision.fallback_used.is_none());
}

#[test]
fn load_config_with_validation_success() {
    let allowed = allowed_action_set();
    let engine =
        load_from_yaml_str(EXAMPLE_YAML, Some(&allowed)).expect("config should validate and load");

    let snapshot = build_snapshot_matching();
    let candidates: Vec<ActionId> = vec![
        "action_kill".to_string(),
        "action_fake_output".to_string(),
        "action_throttle".to_string(),
    ];

    let decision = engine.decide(&snapshot, &candidates);
    assert_eq!(decision.allowed.len(), 2);
}

#[test]
fn load_config_with_validation_failure_unknown_action() {
    // Add an extra action to YAML by hand in memory.
    let yaml = r#"
rules:
  - context_conditions: []
    plugin_states: {}
    system_modifiers: {}
    allowed_responses:
      - action_known
      - action_unknown
    blocked_paths: []
    fallback: []
"#;

    let mut allowed = HashSet::new();
    allowed.insert("action_known".to_string());

    let err = load_from_yaml_str(yaml, Some(&allowed))
        .expect_err("should fail due to unknown action");

    match err {
        LogicalBranchError::UnknownAction { action, .. } => {
            assert_eq!(action, "action_unknown".to_string());
        }
        other => panic!("expected UnknownAction, got: {:?}", other),
    }
}