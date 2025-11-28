use serde::{Deserialize, Serialize};

/// Aggregate metrics for a single scenario replay.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScenarioMetrics {
    /// How many events were actually processed.
    pub events_processed: u64,
    /// How many events produced an outcome that differs
    /// from the baseline_action recorded in the event payload.
    pub divergent_decisions: u64,
    /// How many events caused the decision engine to error.
    pub errors: u64,
}
