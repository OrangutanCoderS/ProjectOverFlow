use graph_evolver::{
    EvolutionConfig, GraphEvolver,
    model::{PluginGraph, PluginNode, SuggestedEdge, TelemetrySummary},
};

fn base_graph() -> PluginGraph {
    PluginGraph {
        nodes: vec![
            PluginNode {
                id: "A".to_string(),
                label: "A".to_string(),
                trust: 1.0,
            },
            PluginNode {
                id: "B".to_string(),
                label: "B".to_string(),
                trust: 1.0,
            },
            PluginNode {
                id: "C".to_string(),
                label: "C".to_string(),
                trust: 1.0,
            },
        ],
        edges: vec![],
    }
}

#[test]
fn proposes_edge_when_thresholds_pass() {
    let cfg = EvolutionConfig {
        min_support: 5,
        max_error_rate: 0.1,
        max_latency_ms: 1000.0,
        max_new_edges: 8,
        ..Default::default()
    };
    let evolver = GraphEvolver::new(cfg);
    let base = base_graph();

    let suggestions = vec![SuggestedEdge {
        from: "A".to_string(),
        to: "B".to_string(),
        stats: TelemetrySummary {
            support_count: 10,
            avg_latency_ms: 50.0,
            error_rate: 0.01,
        },
    }];

    let candidates = evolver.propose_from_suggestions(&base, &suggestions);
    assert_eq!(candidates.len(), 1);

    let candidate = &candidates[0];
    assert!(candidate.graph.has_edge(&"A".to_string(), &"B".to_string()));
    assert!(candidate.score.score > 0.0);
}

#[test]
fn rejects_edge_when_support_too_low() {
    let cfg = EvolutionConfig {
        min_support: 10,
        max_error_rate: 0.1,
        max_latency_ms: 1000.0,
        max_new_edges: 8,
        ..Default::default()
    };
    let evolver = GraphEvolver::new(cfg);
    let base = base_graph();

    let suggestions = vec![SuggestedEdge {
        from: "A".to_string(),
        to: "C".to_string(),
        stats: TelemetrySummary {
            support_count: 1,
            avg_latency_ms: 10.0,
            error_rate: 0.0,
        },
    }];

    let candidates = evolver.propose_from_suggestions(&base, &suggestions);
    assert!(candidates.is_empty());
}

#[test]
fn cap_respects_max_new_edges() {
    let cfg = EvolutionConfig {
        min_support: 1,
        max_error_rate: 1.0,
        max_latency_ms: 10_000.0,
        max_new_edges: 4,
        ..Default::default()
    };
    let evolver = GraphEvolver::new(cfg);
    let base = base_graph();

    let suggestions: Vec<SuggestedEdge> = (0..20)
        .map(|i| SuggestedEdge {
            from: "A".to_string(),
            to: format!("N{i}"),
            stats: TelemetrySummary {
                support_count: 10 + i as u64,
                avg_latency_ms: 100.0,
                error_rate: 0.01,
            },
        })
        .collect();

    let candidates = evolver.propose_from_suggestions(&base, &suggestions);
    assert!(candidates.len() <= 4);
}
