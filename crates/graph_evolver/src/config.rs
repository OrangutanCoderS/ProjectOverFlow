use serde::{Deserialize, Serialize};

/// Tuning knobs for candidate generation and scoring.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionConfig {
    /// Minimum number of observations for a pattern to be considered.
    pub min_support: u64,
    /// Maximum acceptable error rate for a candidate edge.
    pub max_error_rate: f64,
    /// Maximum acceptable average latency in ms.
    pub max_latency_ms: f64,
    /// Hard cap on edges added per evolution cycle.
    pub max_new_edges: usize,
    /// Weight for support in the scoring function.
    pub weight_support: f64,
    /// Weight for error rate in the scoring function.
    pub weight_error: f64,
    /// Weight for latency in the scoring function.
    pub weight_latency: f64,
}

impl Default for EvolutionConfig {
    fn default() -> Self {
        Self {
            min_support: 10,
            max_error_rate: 0.05,
            max_latency_ms: 500.0,
            max_new_edges: 32,
            weight_support: 1.0,
            weight_error: 5.0,
            weight_latency: 0.01,
        }
    }
}
