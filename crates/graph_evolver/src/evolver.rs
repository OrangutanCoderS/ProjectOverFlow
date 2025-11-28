use rand::seq::SliceRandom;

use crate::config::EvolutionConfig;
use crate::error::GraphEvolverError;
use crate::model::{
    EvolutionCandidate, EvolutionOp, EvolutionOpKind, GraphScore, PluginGraph, SuggestedEdge,
    TelemetrySummary,
};

/// Stateless evolution engine.
/// All state is passed in via parameters and the config.
pub struct GraphEvolver {
    config: EvolutionConfig,
}

impl GraphEvolver {
    pub fn new(config: EvolutionConfig) -> Self {
        Self { config }
    }

    pub fn config(&self) -> &EvolutionConfig {
        &self.config
    }

    /// Produce candidate graphs by adding edges suggested by telemetry.
    ///
    /// This is deliberately conservative:
    /// - only adds edges that pass thresholds
    /// - never rewrites/removes existing edges
    /// - hard caps number of added edges per cycle
    pub fn propose_from_suggestions(
        &self,
        base: &PluginGraph,
        suggestions: &[SuggestedEdge],
    ) -> Vec<EvolutionCandidate> {
        let mut eligible: Vec<&SuggestedEdge> = suggestions
            .iter()
            .filter(|s| self.is_edge_eligible(base, &s.stats, &s.from, &s.to))
            .collect();

        // Sort: highest support first, then lowest error.
        eligible.sort_by(|a, b| {
            b.stats
                .support_count
                .cmp(&a.stats.support_count)
                .then_with(|| {
                    a.stats
                        .error_rate
                        .partial_cmp(&b.stats.error_rate)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
        });

        // Shuffle + truncate to avoid always picking the same frontier.
        let mut rng = rand::thread_rng();
        if eligible.len() > self.config.max_new_edges {
            eligible.shuffle(&mut rng);
            eligible.truncate(self.config.max_new_edges);
        }

        eligible
            .into_iter()
            .enumerate()
            .map(|(idx, s)| {
                let mut graph = base.clone();
                let weight = self.derive_weight(&s.stats);
                graph.add_edge(s.from.clone(), s.to.clone(), weight);

                let score = self.score_candidate(&s.stats);

                let op = EvolutionOp {
                    kind: EvolutionOpKind::AddEdge,
                    from: Some(s.from.clone()),
                    to: Some(s.to.clone()),
                    delta_weight: Some(weight),
                    reason: format!(
                        "support={} err={:.4} lat={:.1}ms",
                        s.stats.support_count, s.stats.error_rate, s.stats.avg_latency_ms
                    ),
                };

                EvolutionCandidate {
                    id: format!("add-edge-{}", idx),
                    graph,
                    score,
                    operations: vec![op],
                }
            })
            .collect()
    }

    fn is_edge_eligible(
        &self,
        base: &PluginGraph,
        stats: &TelemetrySummary,
        from: &str,
        to: &str,
    ) -> bool {
        // Already exists -> skip.
        if base.has_edge(&from.to_string(), &to.to_string()) {
            return false;
        }

        // Basic telemetry thresholds.
        if stats.support_count < self.config.min_support {
            return false;
        }

        if stats.error_rate > self.config.max_error_rate {
            return false;
        }

        if stats.avg_latency_ms > self.config.max_latency_ms {
            return false;
        }

        true
    }

    /// Heuristic mapping from telemetry stats to an edge weight.
    fn derive_weight(&self, stats: &TelemetrySummary) -> f64 {
        let base = (stats.support_count as f64).ln_1p();
        let penalty = stats.error_rate * 10.0;
        (base - penalty).max(0.1)
    }

    /// Score a candidate purely from the telemetry driving it.
    fn score_candidate(&self, stats: &TelemetrySummary) -> GraphScore {
        let s = self.config.weight_support * (stats.support_count as f64).ln_1p();
        let e = self.config.weight_error * stats.error_rate;
        let l = self.config.weight_latency * stats.avg_latency_ms;

        GraphScore {
            score: s - e - l,
            support_count: stats.support_count,
            avg_latency_ms: stats.avg_latency_ms,
            error_rate: stats.error_rate,
        }
    }

    /// Optional hook if you ever want to plug in JSON configs or reports.
    pub fn to_json_report(
        &self,
        candidates: &[EvolutionCandidate],
    ) -> Result<String, GraphEvolverError> {
        Ok(serde_json::to_string_pretty(candidates)?)
    }
}
