use crate::error::{GraphMergeError, Result};
use crate::model::{Edge, MergePolicy, MergeReport, NodeId, PluginGraph, PluginNode};
use std::collections::{HashMap, HashSet};

/// Merge `remote` DAG into `local` DAG according to `policy`.
/// Returns new merged graph + a structured MergeReport.
pub fn merge_graphs(
    local: &PluginGraph,
    remote: &PluginGraph,
    policy: &MergePolicy,
) -> Result<(PluginGraph, MergeReport)> {
    // Basic sanity checks before we touch anything.
    validate_graph(local, "local")?;
    validate_graph(remote, "remote")?;

    let mut merged = local.clone();
    let mut report = MergeReport::default();

    // 1) Merge nodes under trust + version constraints
    let mut new_nodes_added = 0usize;

    for remote_node in remote.nodes.values() {
        if remote_node.trust < policy.min_trust {
            report.nodes_skipped_low_trust += 1;
            continue;
        }

        if let Some(existing) = merged.nodes.get(&remote_node.id) {
            // Version comparison for conflicts
            if !policy.allow_overwrite {
                report.conflicts_version += 1;
                continue;
            }

            if !policy.allow_downgrade && remote_node.version < existing.version {
                report.conflicts_version += 1;
                continue;
            }

            // Overwrite allowed
            merged.insert_node(remote_node.clone());
            report.nodes_updated += 1;
        } else {
            // New node: respect max_new_nodes limit if configured
            if let Some(limit) = policy.max_new_nodes {
                if new_nodes_added >= limit {
                    report.nodes_skipped_limit += 1;
                    continue;
                }
            }

            merged.insert_node(remote_node.clone());
            new_nodes_added += 1;
            report.nodes_added += 1;
        }
    }

    // 2) Merge edges, rejecting cycles and invalid references
    let mut new_edges_added = 0usize;

    for edge in &remote.edges {
        // Must only add edges whose endpoints exist in merged graph
        if !merged.contains_node(&edge.from) || !merged.contains_node(&edge.to) {
            report.edges_skipped_missing_node += 1;
            continue;
        }

        if let Some(limit) = policy.max_new_edges {
            if new_edges_added >= limit {
                report.edges_skipped_limit += 1;
                continue;
            }
        }

        // Reject edges that introduce a cycle
        if would_create_cycle(&merged, &edge.from, &edge.to) {
            report.edges_skipped_cycle += 1;
            continue;
        }

        merged.add_edge(edge.clone());
        new_edges_added += 1;
        report.edges_added += 1;
    }

    // Final safety: ensure merged graph is still acyclic
    validate_graph(&merged, "merged")?;

    Ok((merged, report))
}

/// Validate basic DAG invariants: no self-loops, no cycles.
pub fn validate_graph(graph: &PluginGraph, label: &str) -> Result<()> {
    // Self-loop detection
    for e in &graph.edges {
        if e.from == e.to {
            return Err(GraphMergeError::Inconsistent(format!(
                "{label} graph has self-loop on node {id}",
                label = label,
                id = e.from
            )));
        }
    }

    if has_cycle(graph) {
        return Err(GraphMergeError::Inconsistent(format!(
            "{label} graph contains a cycle",
            label = label
        )));
    }

    Ok(())
}

/// Return true if adding edge (from -> to) would create a cycle.
fn would_create_cycle(graph: &PluginGraph, from: &NodeId, to: &NodeId) -> bool {
    if from == to {
        return true;
    }

    // Build adjacency with existing edges + candidate edge.
    let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();

    for e in &graph.edges {
        adj.entry(e.from.as_str())
            .or_default()
            .push(e.to.as_str());
    }

    adj.entry(from.as_str())
        .or_default()
        .push(to.as_str());

    // DFS from 'to' to see if we can reach 'from'.
    let mut stack = vec![to.as_str()];
    let mut visited = HashSet::new();

    while let Some(node) = stack.pop() {
        if !visited.insert(node) {
            continue;
        }
        if node == from.as_str() {
            return true;
        }
        if let Some(neighbors) = adj.get(node) {
            for &next in neighbors {
                stack.push(next);
            }
        }
    }

    false
}

/// Full cycle detection on existing graph (no speculative edge).
fn has_cycle(graph: &PluginGraph) -> bool {
    // Kahn's algorithm for DAG detection
    let mut indegree: HashMap<&str, usize> = HashMap::new();
    let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();

    for id in graph.nodes.keys() {
        indegree.entry(id.as_str()).or_insert(0);
    }

    for e in &graph.edges {
        indegree.entry(e.to.as_str())
            .and_modify(|d| *d += 1)
            .or_insert(1);
        adj.entry(e.from.as_str())
            .or_default()
            .push(e.to.as_str());
    }

    let mut queue: Vec<&str> = indegree
        .iter()
        .filter_map(|(n, &deg)| if deg == 0 { Some(*n) } else { None })
        .collect();

    let mut visited_count = 0usize;

    while let Some(node) = queue.pop() {
        visited_count += 1;
        if let Some(neighbors) = adj.get(node) {
            for &next in neighbors {
                if let Some(d) = indegree.get_mut(next) {
                    *d -= 1;
                    if *d == 0 {
                        queue.push(next);
                    }
                }
            }
        }
    }

    visited_count != indegree.len()
}