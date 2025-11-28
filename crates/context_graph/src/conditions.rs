use serde::{Deserialize, Serialize};

/// Numeric thresholds such as weights, scores, anomaly values.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ThresholdCondition {
    pub min: Option<f32>,
    pub max: Option<f32>,
}

/// Time window for decaying or context-sensitive activation.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TimeWindowCondition {
    pub start_ms: u64,
    pub end_ms: u64,
}

/// Tag constraints for semantic filtering.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TagCondition {
    pub required: Vec<String>,
}

/// Complete set of optional edge conditions.
/// MUST implement Default because graph.rs uses `Default::default()`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EdgeConditionSet {
    pub thresholds: Option<ThresholdCondition>,
    pub time_window: Option<TimeWindowCondition>,
    pub tags: Option<TagCondition>,
}

impl EdgeConditionSet {
    /// Convenience constructor for unconditional edges.
    #[inline]
    pub fn always() -> Self {
        Self {
            thresholds: None,
            time_window: None,
            tags: None,
        }
    }
}