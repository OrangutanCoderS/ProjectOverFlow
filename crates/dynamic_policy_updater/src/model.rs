use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Logical identifier for a policy.
pub type PolicyId = String;

/// Version identifier for a policy.
pub type PolicyVersion = String;

/// Where this candidate came from.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PolicySource {
    Manual,
    Learned,
    Imported,
    Merged,
}

/// Evidence summary for how the candidate behaved in replay/simulations.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CandidateEvidence {
    /// How many simulated incidents the candidate mitigated / improved.
    pub supported_events: u64,

    /// How many incidents got worse or failed under this candidate.
    pub conflicted_events: u64,

    /// Estimated false positives introduced by candidate rules.
    pub false_positives: u64,

    /// How many distinct scenarios / traces it has been tested against.
    pub scenario_count: u64,

    /// Normalized stability score [0.0, 1.0] from replay engine.
    pub stability_score: f32,
}

/// A single atomic change in a policy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyChange {
    /// JSON path-style location of the field (e.g. "rules[3].threshold").
    pub path: String,

    /// How the value changed.
    pub kind: PolicyChangeKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PolicyChangeKind {
    Added {
        new_value: Value,
    },
    Removed {
        old_value: Value,
    },
    Modified {
        old_value: Value,
        new_value: Value,
    },
}

/// Structured diff between base and candidate policies.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PolicyDiff {
    pub base_version: Option<PolicyVersion>,
    pub candidate_version: PolicyVersion,
    pub changes: Vec<PolicyChange>,
}

/// Score breakdown for a candidate.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PolicyScore {
    /// Effectiveness / improvement score [0.0, 1.0].
    pub benefit_score: f32,

    /// Risk / regressions score [0.0, 1.0] (higher = more risky).
    pub risk_score: f32,

    /// Confidence in the estimate [0.0, 1.0].
    pub confidence: f32,

    /// Aggregated decision score [0.0, 1.0].
    pub overall: f32,
}

/// Recommended action. Note: this crate never executes it.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RecommendationAction {
    /// Candidate looks safe and beneficial, but still requires explicit apply.
    SafeToApply,

    /// Candidate is interesting but not strong enough to apply.
    ReviewOnly,

    /// Candidate looks harmful or too uncertain.
    Reject,
}

/// Candidate policy + all context needed for scoring.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidatePolicy {
    pub id: PolicyId,

    /// Version of the currently active base policy (if known).
    pub base_version: Option<PolicyVersion>,

    /// Proposed candidate version identifier.
    pub candidate_version: PolicyVersion,

    pub source: PolicySource,

    /// Evidence summary provided by replay/pattern engines.
    pub evidence: CandidateEvidence,

    /// JSON of the base policy (before change).
    pub base_policy: Value,

    /// JSON of the candidate policy (after change).
    pub candidate_policy: Value,
}

/// Final recommendation emitted by the updater.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyUpdateRecommendation {
    pub id: PolicyId,
    pub base_version: Option<PolicyVersion>,
    pub candidate_version: PolicyVersion,
    pub diff: PolicyDiff,
    pub score: PolicyScore,
    pub action: RecommendationAction,
    pub rationale: String,
}