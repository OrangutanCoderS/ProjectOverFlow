use criterion::{black_box, criterion_group, criterion_main, Criterion};
use policy_tree::{
    ConditionKind, ConditionSet, EvalContext, MatchMode, PolicyActionDef, PolicyNodeDef,
    PolicyTree, PolicyTreeDef,
};
use std::collections::{HashMap, HashSet};

fn build_sample_tree() -> PolicyTree {
    // A slightly wider tree to get non-trivial cost.
    let mut nodes = Vec::new();

    nodes.push(PolicyNodeDef {
        id: "root".to_string(),
        conditions: None,
        children: vec!["n1".to_string(), "n2".to_string(), "n3".to_string()],
        action: None,
    });

    for i in 1..=3 {
        let id = format!("n{i}");
        nodes.push(PolicyNodeDef {
            id: id.clone(),
            conditions: Some(ConditionSet {
                all: vec![ConditionKind::Threshold {
                    metric: "score".to_string(),
                    min: Some(i as f32),
                    max: None,
                }],
                any: Vec::new(),
            }),
            children: Vec::new(),
            action: Some(PolicyActionDef {
                name: format!("Action{i}"),
                severity: i as u8,
                labels: vec!["bench".to_string()],
            }),
        });
    }

    let def = PolicyTreeDef {
        root_id: "root".to_string(),
        nodes,
    };

    PolicyTree::from_def(def).expect("tree builds in bench")
}

fn bench_policy_eval(c: &mut Criterion) {
    let tree = build_sample_tree();

    let mut metrics = HashMap::new();
    metrics.insert("score".to_string(), 2.5);
    let tags: HashSet<String> = HashSet::new();

    let ctx = EvalContext {
        metrics: &metrics,
        tags: &tags,
        timestamp_ms: 42,
    };

    c.bench_function("policy_tree_eval_collect_all", |b| {
        b.iter(|| {
            let res = tree.evaluate(black_box(&ctx), MatchMode::CollectAll);
            black_box(res);
        });
    });
}

criterion_group!(benches, bench_policy_eval);
criterion_main!(benches);