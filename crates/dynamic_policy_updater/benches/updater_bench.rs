use criterion::{black_box, criterion_group, criterion_main, Criterion};
use dynamic_policy_updater::{
    CandidateEvidence, CandidatePolicy, DynamicPolicyUpdater, PolicyScoreConfig, PolicySource,
};
use serde_json::json;

fn build_updater() -> DynamicPolicyUpdater {
    let scorer = dynamic_policy_updater::PolicyScorer::new(PolicyScoreConfig::default());
    DynamicPolicyUpdater::new(scorer)
}

fn bench_updater(c: &mut Criterion) {
    let updater = build_updater();

    let base = json!({
        "rules": (0..50).map(|i| json!({
            "id": format!("r{}", i),
            "threshold": 10 + i
        })).collect::<Vec<_>>()
    });

    let candidate = json!({
        "rules": (0..50).map(|i| json!({
            "id": format!("r{}", i),
            "threshold": 5 + i   // tweak thresholds
        })).collect::<Vec<_>>()
    });

    let evidence = CandidateEvidence {
        supported_events: 1_000,
        conflicted_events: 50,
        false_positives: 25,
        scenario_count: 200,
        stability_score: 0.85,
    };

    c.bench_function("dynamic_policy_updater_evaluate", |b| {
        b.iter(|| {
            let candidate_policy = CandidatePolicy {
                id: "bench-policy".to_string(),
                base_version: Some("v1".to_string()),
                candidate_version: "v2".to_string(),
                source: PolicySource::Learned,
                evidence: evidence.clone(),
                base_policy: base.clone(),
                candidate_policy: candidate.clone(),
            };

            let _ = updater.evaluate(black_box(candidate_policy)).unwrap();
        });
    });
}

criterion_group!(benches, bench_updater);
criterion_main!(benches);