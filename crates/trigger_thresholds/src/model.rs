use serde::{Deserialize, Serialize};

/// One threshold rule (warn/block)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdRule {
    pub warn: f64,
    pub block: f64,
    #[serde(default = "default_min_safe")]
    pub min_safe: f64,
    #[serde(default = "default_max_safe")]
    pub max_safe: f64,
}

fn default_min_safe() -> f64 { 0.0 }
fn default_max_safe() -> f64 { 100.0 }

/// Full threshold configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdConfig {
    pub entropy: ThresholdRule,
    pub cpu: ThresholdRule,
    pub net_spike_kb: ThresholdRule,
    #[serde(default)]
    pub adaptive: bool,
}