use std::collections::HashMap;

use dynamic_policy_updater::{
    CandidateEvidence, CandidatePolicy, DynamicPolicyUpdater, PolicyScoreConfig, PolicySource,
    RecommendationAction,
};
use serde_json::json;

fn build_updater() -> DynamicPolicyUpdater {
    let scorer = dynamic_policy_updater::PolicyScorer::new(PolicyScoreConfig::default());
    DynamicPolicyUpdater::new(scorer)
}

#[test]
fn diff_detects_added_and_modified_fields() {
    let base = json!({
        "rules": [
            { "id": "r1", "threshold": 10 },
        ],
        "meta": { "version": 1 }
    });

    let candidate = json!({
        "rules": [
            { "id": "r1", "threshold": 5 },
            { "id": "r2", "threshold": 20 }
        ],
        "meta": { "version": 2 }
    });

    let candidate_policy = CandidatePolicy {
        id: "policy-1".to_string(),
        base_version: Some("v1".to_string()),
        candidate_version: "v2".to_string(),
        source: PolicySource::Learned,
        evidence: CandidateEvidence {
            supported_events: 10,
            conflicted_events: 1,
            false_positives: 1,
            scenario_count: 12,
            stability_score: 0.8,
        },
        base_policy: base,
        candidate_policy: candidate,
    };

    let updater = build_updater();
    let recommendation = updater.evaluate(candidate_policy).unwrap();

    // Ensure we detected at least one change.
    assert!(!recommendation.diff.changes.is_empty());

    // Check that some path we expect appears.
    let paths: HashMap<_, _> = recommendation
        .diff
        .changes
        .iter()
        .map(|c| (c.path.clone(), &c.kind))
        .collect();

    assert!(paths.keys().any(|p| p.contains("rules[0].threshold")));
    assert!(paths.keys().any(|p| p.contains("rules[1]")));
}

#[test]
fn scoring_can_produce_safe_to_apply() {
    let base = json!({ "rules": [] });
    let candidate = json!({ "rules": [{"id": "r1", "threshold": 10 }] });

    let candidate_policy = CandidatePolicy {
        id: "policy-2".to_string(),
        base_version: None,
        candidate_version: "v1".to_string(),
        source: PolicySource::Manual,
        evidence: CandidateEvidence {
            supported_events: 90,
            conflicted_events: 3,
            false_positives: 3,
            scenario_count: 50,
            stability_score: 0.9,
        },
        base_policy: base,
        candidate_policy: candidate,
    };

    let updater = build_updater();
    let recommendation = updater.evaluate(candidate_policy).unwrap();

    assert!(
        recommendation.action == RecommendationAction::SafeToApply
            || recommendation.action == RecommendationAction::ReviewOnly
    );
}

#[test]
fn scoring_can_reject_bad_candidate() {
    let base = json!({ "rules": [{"id": "r1", "threshold": 10 }] });
    let candidate = json!({ "rules": [{"id": "r1", "threshold": 1 }] });

    let candidate_policy = CandidatePolicy {
        id: "policy-3".to_string(),
        base_version: Some("v1".to_string()),
        candidate_version: "v2".to_string(),
        source: PolicySource::Learned,
        evidence: CandidateEvidence {
            supported_events: 1,
            conflicted_events: 20,
            false_positives: 20,
            scenario_count: 5,
            stability_score: 0.2,
        },
        base_policy: base,
        candidate_policy: candidate,
    };

    let updater = build_updater();
    let recommendation = updater.evaluate(candidate_policy).unwrap();

    assert_eq!(recommendation.action, RecommendationAction::Reject);
}