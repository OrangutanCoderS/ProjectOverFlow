use policy_tree::{
    ConditionKind, ConditionSet, EvalContext, MatchMode, PolicyActionDef, PolicyNodeDef,
    PolicyTree, PolicyTreeDef,
};
use std::collections::{HashMap, HashSet};

#[test]
fn build_and_eval_simple_tree() {
    // root -> child; child has a threshold condition and an action.
    let def = PolicyTreeDef {
        root_id: "root".to_string(),
        nodes: vec![
            PolicyNodeDef {
                id: "root".to_string(),
                conditions: None,
                children: vec!["child".to_string()],
                action: None,
            },
            PolicyNodeDef {
                id: "child".to_string(),
                conditions: Some(ConditionSet {
                    all: vec![ConditionKind::Threshold {
                        metric: "cpu".to_string(),
                        min: Some(0.8),
                        max: None,
                    }],
                    any: Vec::new(),
                }),
                children: Vec::new(),
                action: Some(PolicyActionDef {
                    name: "HighCPU".to_string(),
                    severity: 5,
                    labels: vec!["cpu".to_string(), "alert".to_string()],
                }),
            },
        ],
    };

    let tree = PolicyTree::from_def(def).expect("tree should build");

    // Context that should match.
    let mut metrics = HashMap::new();
    metrics.insert("cpu".to_string(), 0.9);
    let tags: HashSet<String> = HashSet::new();

    let ctx = EvalContext {
        metrics: &metrics,
        tags: &tags,
        timestamp_ms: 1234,
    };

    let matches = tree.evaluate(&ctx, MatchMode::CollectAll);
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].node_id, "child");
    assert_eq!(matches[0].action.name, "HighCPU");

    // Context that should NOT match.
    let mut metrics_low = HashMap::new();
    metrics_low.insert("cpu".to_string(), 0.5);

    let ctx_low = EvalContext {
        metrics: &metrics_low,
        tags: &tags,
        timestamp_ms: 1234,
    };

    let matches_low = tree.evaluate(&ctx_low, MatchMode::CollectAll);
    assert!(matches_low.is_empty());
}

#[test]
fn detects_cycle_in_definition() {
    let def = PolicyTreeDef {
        root_id: "a".to_string(),
        nodes: vec![
            PolicyNodeDef {
                id: "a".to_string(),
                conditions: None,
                children: vec!["b".to_string()],
                action: None,
            },
            PolicyNodeDef {
                id: "b".to_string(),
                conditions: None,
                children: vec!["a".to_string()], // cycle back to a
                action: None,
            },
        ],
    };

    let err = PolicyTree::from_def(def).expect_err("cycle must be detected");
    let msg = format!("{err}");
    assert!(msg.contains("cycle"), "unexpected error: {msg}");
}