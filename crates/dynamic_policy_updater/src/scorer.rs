use crate::model::{CandidateEvidence, PolicyScore, RecommendationAction};

/// Configurable weights for scoring.
#[derive(Debug, Clone)]
pub struct PolicyScoreConfig {
    /// Weight for benefit in overall score.
    pub benefit_weight: f32,

    /// Weight for risk contribution (subtracted).
    pub risk_weight: f32,

    /// Weight for confidence.
    pub confidence_weight: f32,

    /// Minimum overall score to consider "SafeToApply".
    pub safe_threshold: f32,

    /// Minimum overall score to not reject outright.
    pub review_threshold: f32,
}

impl Default for PolicyScoreConfig {
    fn default() -> Self {
        Self {
            benefit_weight: 0.5,
            risk_weight: 0.3,
            confidence_weight: 0.2,
            safe_threshold: 0.8,
            review_threshold: 0.4,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PolicyScorer {
    cfg: PolicyScoreConfig,
}

impl PolicyScorer {
    pub fn new(cfg: PolicyScoreConfig) -> Self {
        Self { cfg }
    }

    pub fn config(&self) -> &PolicyScoreConfig {
        &self.cfg
    }

    /// Score a candidate from its evidence.
    pub fn score(&self, ev: &CandidateEvidence) -> PolicyScore {
        let total_events = ev.supported_events + ev.conflicted_events + ev.false_positives;
        let total_events_f = total_events as f32;

        // Benefit: how often it helped vs hurt.
        let benefit_score = if total_events > 0 {
            (ev.supported_events as f32 / total_events_f)
                .min(1.0)
                .max(0.0)
        } else {
            0.0
        };

        // Risk: conflicted + false positives normalized.
        let risk_score = if total_events > 0 {
            ((ev.conflicted_events + ev.false_positives) as f32 / total_events_f)
                .min(1.0)
                .max(0.0)
        } else {
            0.0
        };

        // Confidence: scenarios + stability.
        let scenario_factor = (ev.scenario_count as f32 / 50.0).min(1.0); // saturate at ~50 scenarios
        let stability = ev.stability_score.clamp(0.0, 1.0);
        let confidence = (scenario_factor * 0.6 + stability * 0.4).clamp(0.0, 1.0);

        // Aggregate
        let overall = (self.cfg.benefit_weight * benefit_score)
            - (self.cfg.risk_weight * risk_score)
            + (self.cfg.confidence_weight * confidence);

        let overall = overall.clamp(0.0, 1.0);

        PolicyScore {
            benefit_score,
            risk_score,
            confidence,
            overall,
        }
    }

    /// Decide a recommended action from the score.
    pub fn decide_action(&self, score: &PolicyScore) -> RecommendationAction {
        if score.overall >= self.cfg.safe_threshold && score.risk_score < 0.3 {
            RecommendationAction::SafeToApply
        } else if score.overall >= self.cfg.review_threshold {
            RecommendationAction::ReviewOnly
        } else {
            RecommendationAction::Reject
        }
    }

    /// Build a human-readable rationale string.
    pub fn build_rationale(&self, score: &PolicyScore) -> String {
        format!(
            "benefit={:.2}, risk={:.2}, confidence={:.2}, overall={:.2}",
            score.benefit_score, score.risk_score, score.confidence, score.overall
        )
    }
}