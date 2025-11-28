use criterion::{black_box, criterion_group, criterion_main, Criterion};
use graph_evolver::{
    EvolutionConfig, GraphEvolver,
    model::{PluginGraph, PluginNode, SuggestedEdge, TelemetrySummary},
};

fn build_graph(num_nodes: usize) -> PluginGraph {
    let mut nodes = Vec::with_capacity(num_nodes);
    for i in 0..num_nodes {
        nodes.push(PluginNode {
            id: format!("N{i}"),
            label: format!("Node {i}"),
            trust: 1.0,
        });
    }

    PluginGraph {
        nodes,
        edges: Vec::new(),
    }
}

fn build_suggestions(num: usize) -> Vec<SuggestedEdge> {
    let mut out = Vec::with_capacity(num);
    for i in 0..num {
        let from = format!("N{}", i % 50);
        let to = format!("N{}", (i + 1) % 50);

        out.push(SuggestedEdge {
            from,
            to,
            stats: TelemetrySummary {
                support_count: 10 + (i as u64 % 100),
                avg_latency_ms: 50.0 + (i as f64 % 20.0),
                error_rate: 0.01 + ((i as f64 % 5.0) * 0.001),
            },
        });
    }
    out
}

fn bench_evolution(c: &mut Criterion) {
    let cfg = EvolutionConfig::default();
    let evolver = GraphEvolver::new(cfg);
    let base = build_graph(100);
    let suggestions = build_suggestions(1_000);

    c.bench_function("graph_evolver_propose_1000", |b| {
        b.iter(|| {
            let _candidates =
                evolver.propose_from_suggestions(black_box(&base), black_box(&suggestions));
        });
    });
}

criterion_group!(benches, bench_evolution);
criterion_main!(benches);
