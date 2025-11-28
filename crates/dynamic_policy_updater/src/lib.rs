//! Dynamic Policy Updater (Phase V, diff-only).
//!
//! This crate does **not** auto-apply policies.
//! It:
//! - computes diffs between a base policy JSON and a candidate policy JSON
//! - scores the candidate using evidence
//! - emits a `PolicyUpdateRecommendation` for higher layers / humans.

mod model;
mod scorer;
mod diff;
mod engine;

pub use crate::model::{
    CandidateEvidence,
    CandidatePolicy,
    PolicyChange,
    PolicyChangeKind,
    PolicyDiff,
    PolicyId,
    PolicyScore,
    PolicySource,
    PolicyUpdateRecommendation,
    PolicyVersion,
    RecommendationAction,
};

pub use crate::scorer::{PolicyScoreConfig, PolicyScorer};
pub use crate::diff::diff_policies;
pub use crate::engine::DynamicPolicyUpdater;

/// Crate-level error type.
pub type Result<T> = std::result::Result<T, DynamicPolicyError>;

/// Errors emitted by the updater.
#[derive(Debug, thiserror::Error)]
pub enum DynamicPolicyError {
    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("invalid input: {0}")]
    InvalidInput(String),
}