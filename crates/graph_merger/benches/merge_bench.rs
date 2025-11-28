use criterion::{black_box, criterion_group, criterion_main, Criterion};
use graph_merger::{merge_graphs, Edge, MergePolicy, PluginGraph, PluginNode};
use rand::distributions::Alphanumeric;
use rand::{rngs::StdRng, Rng, SeedableRng};
use serde_json::json;

/// Generate a random DAG with N nodes and ~E edges.
/// Edges only go from lower index -> higher index to keep DAG property.
fn make_random_dag(nodes: usize, edges: usize, rng: &mut StdRng) -> PluginGraph {
    let mut graph = PluginGraph::with_capacity(nodes, edges);

    // Create nodes
    for i in 0..nodes {
        let id = format!("n{i}");
        let plugin_id: String = rng
            .sample_iter(Alphanumeric)
            .take(8)
            .map(char::from)
            .collect();

        let node = PluginNode {
            id: id.clone(),
            plugin_id,
            version: 1,
            trust: 0.9,
            tags: vec!["bench".to_string()],
            metadata: json!({ "idx": i }),
        };

        graph.insert_node(node);
    }

    // Create edges (only forward edges to avoid cycles)
    let node_ids: Vec<String> = graph.nodes.keys().cloned().collect();
    let n = node_ids.len();

    for _ in 0..edges {
        let from_idx = rng.gen_range(0..n.saturating_sub(1));
        let to_idx = rng.gen_range((from_idx + 1)..n);

        graph.add_edge(Edge {
            from: node_ids[from_idx].clone(),
            to: node_ids[to_idx].clone(),
            label: None,
        });
    }

    graph
}

fn bench_merge_dense(c: &mut Criterion) {
    let mut rng = StdRng::seed_from_u64(42);

    // Local graph ~1000 nodes, 3000 edges
    let local = make_random_dag(1000, 3000, &mut rng);
    // Remote graph ~300 nodes, 1000 edges
    let remote = make_random_dag(300, 1000, &mut rng);

    let policy = MergePolicy::default();

    c.bench_function("graph_merge_1000x300_dense", |b| {
        b.iter(|| {
            let (merged, report) = merge_graphs(black_box(&local), black_box(&remote), &policy)
                .expect("merge should succeed");

            // Prevent compiler from optimizing away
            black_box(merged);
            black_box(report);
        });
    });
}

criterion_group!(benches, bench_merge_dense);
criterion_main!(benches);