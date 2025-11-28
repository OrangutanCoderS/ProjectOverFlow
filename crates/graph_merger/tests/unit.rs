use graph_merger::{
    merge_graphs, validate_graph, Edge, MergePolicy, PluginGraph, PluginNode,
};

fn make_node(id: &str, version: u32, trust: f32) -> PluginNode {
    PluginNode {
        id: id.to_string(),
        plugin_id: format!("plugin-{id}"),
        version,
        trust,
        tags: vec![],
        metadata: serde_json::json!({}),
    }
}

#[test]
fn merge_adds_new_nodes_and_edges() {
    let mut local = PluginGraph::new();
    local.insert_node(make_node("A", 1, 1.0));

    let mut remote = PluginGraph::new();
    remote.insert_node(make_node("B", 1, 0.9));
    remote.add_edge(Edge {
        from: "A".into(),
        to: "B".into(),
        label: None,
    });

    let policy = MergePolicy::default();
    let (merged, report) = merge_graphs(&local, &remote, &policy).unwrap();

    assert!(merged.contains_node("A"));
    assert!(merged.contains_node("B"));
    assert_eq!(report.nodes_added, 1);
    assert_eq!(report.edges_added, 1);

    validate_graph(&merged, "merged").unwrap();
}

#[test]
fn low_trust_nodes_are_rejected() {
    let mut local = PluginGraph::new();
    local.insert_node(make_node("A", 1, 1.0));

    let mut remote = PluginGraph::new();
    // trust below default min_trust = 0.5
    remote.insert_node(make_node("B", 1, 0.2));
    remote.add_edge(Edge {
        from: "A".into(),
        to: "B".into(),
        label: None,
    });

    let policy = MergePolicy::default();
    let (merged, report) = merge_graphs(&local, &remote, &policy).unwrap();

    assert!(!merged.contains_node("B"));
    assert_eq!(report.nodes_skipped_low_trust, 1);
    assert_eq!(report.edges_added, 0);
}

#[test]
fn cycle_edges_are_rejected() {
    // Local: A -> B
    let mut local = PluginGraph::new();
    local.insert_node(make_node("A", 1, 1.0));
    local.insert_node(make_node("B", 1, 1.0));
    local.add_edge(Edge {
        from: "A".into(),
        to: "B".into(),
        label: None,
    });

    // Remote wants B -> A, which would create a cycle.
    let mut remote = PluginGraph::new();
    remote.insert_node(make_node("A", 2, 1.0));
    remote.insert_node(make_node("B", 2, 1.0));
    remote.add_edge(Edge {
        from: "B".into(),
        to: "A".into(),
        label: None,
    });

    let policy = MergePolicy::default();
    let (merged, report) = merge_graphs(&local, &remote, &policy).unwrap();

    assert_eq!(report.edges_skipped_cycle, 1);
    // Local edge still exists.
    assert_eq!(merged.edges.len(), 1);
    validate_graph(&merged, "merged").unwrap();
}

#[test]
fn version_conflicts_respected() {
    let mut local = PluginGraph::new();
    local.insert_node(make_node("A", 5, 1.0));

    let mut remote = PluginGraph::new();
    // lower version trying to overwrite
    remote.insert_node(make_node("A", 3, 0.9));

    let mut policy = MergePolicy::default();
    policy.allow_overwrite = true;
    policy.allow_downgrade = false;

    let (merged, report) = merge_graphs(&local, &remote, &policy).unwrap();
    let merged_a = merged.nodes.get("A").unwrap();

    assert_eq!(merged_a.version, 5);
    assert_eq!(report.conflicts_version, 1);
}

#[test]
fn downgrade_allowed_when_configured() {
    let mut local = PluginGraph::new();
    local.insert_node(make_node("A", 5, 1.0));

    let mut remote = PluginGraph::new();
    remote.insert_node(make_node("A", 3, 0.9));

    let mut policy = MergePolicy::default();
    policy.allow_overwrite = true;
    policy.allow_downgrade = true;

    let (merged, report) = merge_graphs(&local, &remote, &policy).unwrap();
    let merged_a = merged.nodes.get("A").unwrap();

    assert_eq!(merged_a.version, 3);
    assert_eq!(report.nodes_updated, 1);
}

#[test]
fn node_and_edge_limits_are_enforced() {
    let mut local = PluginGraph::new();
    local.insert_node(make_node("A", 1, 1.0));

    let mut remote = PluginGraph::new();
    remote.insert_node(make_node("B", 1, 0.9));
    remote.insert_node(make_node("C", 1, 0.9)); // exceeds limit later

    remote.add_edge(Edge {
        from: "A".into(),
        to: "B".into(),
        label: None,
    });
    remote.add_edge(Edge {
        from: "B".into(),
        to: "C".into(),
        label: None,
    });

    let mut policy = MergePolicy::default();
    policy.max_new_nodes = Some(1);
    policy.max_new_edges = Some(1);

    let (merged, report) = merge_graphs(&local, &remote, &policy).unwrap();

    assert!(merged.contains_node("B"));
    assert!(!merged.contains_node("C"));
    assert_eq!(report.nodes_added, 1);
    assert_eq!(report.nodes_skipped_limit, 1);
    assert_eq!(report.edges_added, 1);
    assert_eq!(report.edges_skipped_limit, 1);
}