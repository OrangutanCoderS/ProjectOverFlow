use crate::error::MemoryReplayError;
use crate::metrics::ScenarioMetrics;
use crate::model::{
    DecisionOutcome,
    ReplayEvent,
    ScenarioConfig,
    ScenarioId,
};
use serde_json::Value;
use std::sync::Arc;

/// Abstract decision engine used by the replay engine.
///
/// In Phase V you can back this with plugin_graph_runtime,
/// trigger_policies, or any other OverFlow decision core.
pub trait DecisionEngine: Send + Sync {
    fn evaluate(&self, event: &ReplayEvent) -> DecisionOutcome;
}

/// A single scenario binding a decision engine and config.
#[derive(Debug, Clone)]
pub struct ScenarioRunner {
    pub id: ScenarioId,
    pub config: ScenarioConfig,
    pub engine: Arc<dyn DecisionEngine>,
}

/// Per-event record inside a scenario result.
#[derive(Debug, Clone)]
pub struct DecisionRecord {
    pub event_index: usize,
    pub event_seq: u64,
    pub outcome: DecisionOutcome,
    pub divergent: bool,
}

/// Full replay result for a scenario.
#[derive(Debug, Clone)]
pub struct ScenarioResult {
    pub id: ScenarioId,
    pub config: ScenarioConfig,
    pub metrics: ScenarioMetrics,
    pub decisions: Vec<DecisionRecord>,
}

/// The memory replay engine: deterministic iteration over a
/// recorded timeline, passing each event into a decision engine
/// to simulate alternate outcomes.
pub struct MemoryReplayEngine {
    events: Vec<ReplayEvent>,
}

impl MemoryReplayEngine {
    /// Construct a replay engine from a set of events.
    ///
    /// Events are sorted by (timeline_id, sequence_no, timestamp).
    /// If the list is empty, returns `MemoryReplayError::EmptyTimeline`.
    pub fn new(mut events: Vec<ReplayEvent>) -> Result<Self, MemoryReplayError> {
        if events.is_empty() {
            return Err(MemoryReplayError::EmptyTimeline);
        }

        events.sort_by(|a, b| {
            a.timeline_id
                .cmp(&b.timeline_id)
                .then_with(|| a.sequence_no.cmp(&b.sequence_no))
                .then_with(|| a.timestamp.cmp(&b.timestamp))
        });

        // Optional: sanity check ordering invariants.
        for (idx, pair) in events.windows(2).enumerate() {
            let a = &pair[0];
            let b = &pair[1];
            if a.timeline_id > b.timeline_id {
                return Err(MemoryReplayError::InvalidOrder { index: idx });
            }
        }

        Ok(Self { events })
    }

    /// Access underlying events (read-only).
    pub fn events(&self) -> &[ReplayEvent] {
        &self.events
    }

    /// Run a single scenario over the entire timeline (or up to
    /// max_events if configured).
    pub fn run_scenario(&self, runner: &ScenarioRunner) -> ScenarioResult {
        let limit = runner.config.max_events.unwrap_or(self.events.len());
        let total = self.events.len().min(limit);

        let mut metrics = ScenarioMetrics::default();
        let mut decisions = Vec::with_capacity(total);

        for (idx, ev) in self.events.iter().take(total).enumerate() {
            let outcome = runner.engine.evaluate(ev);

            metrics.events_processed += 1;

            let divergent = is_divergent(ev, &outcome);
            if divergent {
                metrics.divergent_decisions += 1;
            }

            decisions.push(DecisionRecord {
                event_index: idx,
                event_seq: ev.sequence_no,
                outcome,
                divergent,
            });
        }

        ScenarioResult {
            id: runner.id.clone(),
            config: runner.config.clone(),
            metrics,
            decisions,
        }
    }

    /// Run multiple scenarios against the same timeline.
    ///
    /// Each scenario is executed independently with its own engine.
    pub fn run_scenarios(&self, runners: &[ScenarioRunner]) -> Vec<ScenarioResult> {
        runners.iter().map(|r| self.run_scenario(r)).collect()
    }
}

/// Determine if the engine's outcome diverges from the baseline
/// recorded at capture time.
///
/// By convention we look for `payload["baseline_action"]` and
/// compare to `outcome.action`.
fn is_divergent(event: &ReplayEvent, outcome: &DecisionOutcome) -> bool {
    match event.payload.get("baseline_action") {
        Some(Value::String(baseline)) => baseline != &outcome.action,
        _ => false,
    }
}
