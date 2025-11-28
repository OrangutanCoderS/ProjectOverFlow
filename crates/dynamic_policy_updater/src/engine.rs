use crate::diff::diff_policies;
use crate::model::{
    CandidatePolicy, PolicyScore, PolicyUpdateRecommendation, RecommendationAction,
};
use crate::{DynamicPolicyError, PolicyScorer, Result};

/// Main orchestrator: given a candidate, compute diff+score+recommendation.
#[derive(Debug, Clone)]
pub struct DynamicPolicyUpdater {
    scorer: PolicyScorer,
}

impl DynamicPolicyUpdater {
    pub fn new(scorer: PolicyScorer) -> Self {
        Self { scorer }
    }

    /// Evaluate the candidate and produce a recommendation.
    ///
    /// This function does **not** modify any external state.
    pub fn evaluate(&self, candidate: CandidatePolicy) -> Result<PolicyUpdateRecommendation> {
        // Diff base vs candidate JSON.
        let diff = diff_policies(
            candidate.base_version.clone(),
            candidate.candidate_version.clone(),
            &candidate.base_policy,
            &candidate.candidate_policy,
        );

        // Score from evidence.
        let score: PolicyScore = self.scorer.score(&candidate.evidence);

        // Decide action – advisory only.
        let action: RecommendationAction = self.scorer.decide_action(&score);
        let rationale = self.scorer.build_rationale(&score);

        Ok(PolicyUpdateRecommendation {
            id: candidate.id,
            base_version: candidate.base_version,
            candidate_version: candidate.candidate_version,
            diff,
            score,
            action,
            rationale,
        })
    }
}