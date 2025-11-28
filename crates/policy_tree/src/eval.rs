use crate::error::PolicyTreeError;
use crate::model::{
    ConditionKind, ConditionSet, PolicyActionDef, PolicyNodeDef, PolicyTreeDef,
    MAX_POLICY_NODES,
};
use std::collections::{HashMap, HashSet};

/// Runtime node representation.
#[derive(Debug, Clone)]
pub struct PolicyNode {
    pub id: String,
    /// Conditions to enter this node (empty = always true).
    pub conditions: ConditionSet,
    /// Indices of child nodes in `PolicyTree.nodes`.
    pub children: Vec<usize>,
    /// Optional leaf action.
    pub action: Option<PolicyActionDef>,
}

/// Policy tree evaluation mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatchMode {
    /// Stop on first matching leaf (DFS order).
    FirstMatch,
    /// Collect all matching leaves reachable from root.
    CollectAll,
}

/// Evaluation context: what the tree matches against.
#[derive(Debug)]
pub struct EvalContext<'a> {
    pub metrics: &'a HashMap<String, f32>,
    pub tags: &'a HashSet<String>,
    pub timestamp_ms: u64,
}

/// Result of a policy match.
#[derive(Debug)]
pub struct PolicyMatch<'a> {
    pub node_id: &'a str,
    pub action: &'a PolicyActionDef,
}

/// Runtime policy tree.
#[derive(Debug)]
pub struct PolicyTree {
    nodes: Vec<PolicyNode>,
    root: usize,
}

impl PolicyTree {
    /// Build and validate a tree from the config definition.
    ///
    /// Complexity: O(N + E) for N nodes and E edges.
    pub fn from_def(def: PolicyTreeDef) -> Result<Self, PolicyTreeError> {
        if def.nodes.len() > MAX_POLICY_NODES {
            return Err(PolicyTreeError::NodeLimitExceeded {
                attempted: def.nodes.len(),
                max: MAX_POLICY_NODES,
            });
        }

        // 1) Map ids -> index and detect duplicates.
        let mut id_to_index: HashMap<String, usize> = HashMap::with_capacity(def.nodes.len());
        for (idx, n) in def.nodes.iter().enumerate() {
            if id_to_index.insert(n.id.clone(), idx).is_some() {
                return Err(PolicyTreeError::DuplicateNodeId(n.id.clone()));
            }
        }

        // 2) Resolve root.
        let root = match id_to_index.get(&def.root_id) {
            Some(&idx) => idx,
            None => return Err(PolicyTreeError::UnknownRootId(def.root_id)),
        };

        // 3) Build runtime nodes with child indices.
        let mut nodes: Vec<PolicyNode> = Vec::with_capacity(def.nodes.len());

        // First create nodes with empty children; then we fill children.
        for n in &def.nodes {
            let conditions = n.conditions.clone().unwrap_or_default();
            nodes.push(PolicyNode {
                id: n.id.clone(),
                conditions,
                children: Vec::new(),
                action: n.action.clone(),
            });
        }

        // Resolve children.
        for n in &def.nodes {
            let parent_idx = id_to_index[&n.id];
            let mut child_indices = Vec::with_capacity(n.children.len());
            for child_id in &n.children {
                let child_idx = match id_to_index.get(child_id) {
                    Some(&idx) => idx,
                    None => {
                        return Err(PolicyTreeError::UnknownChildId {
                            parent_id: n.id.clone(),
                            child_id: child_id.clone(),
                        })
                    }
                };
                child_indices.push(child_idx);
            }
            nodes[parent_idx].children = child_indices;
        }

        // 4) Cycle detection (defensive: tree should be acyclic).
        let mut visited = vec![false; nodes.len()];
        let mut stack = vec![false; nodes.len()];

        fn dfs(
            u: usize,
            nodes: &Vec<PolicyNode>,
            visited: &mut [bool],
            stack: &mut [bool],
        ) -> Result<(), PolicyTreeError> {
            if stack[u] {
                return Err(PolicyTreeError::CycleDetected(nodes[u].id.clone()));
            }
            if visited[u] {
                return Ok(());
            }

            visited[u] = true;
            stack[u] = true;

            for &v in &nodes[u].children {
                dfs(v, nodes, visited, stack)?;
            }

            stack[u] = false;
            Ok(())
        }

        dfs(root, &nodes, &mut visited, &mut stack)?;

        Ok(PolicyTree { nodes, root })
    }

    /// Evaluate the tree from root against a context.
    ///
    /// Complexity: O(number_of_nodes_visited * average_conditions_per_node)
    pub fn evaluate<'a>(
        &'a self,
        ctx: &EvalContext<'a>,
        mode: MatchMode,
    ) -> Vec<PolicyMatch<'a>> {
        let mut results = Vec::new();
        let mut stop = false;

        fn eval_node<'a>(
            tree: &'a PolicyTree,
            idx: usize,
            ctx: &EvalContext<'a>,
            mode: MatchMode,
            results: &mut Vec<PolicyMatch<'a>>,
            stop: &mut bool,
        ) {
            if *stop {
                return;
            }

            let node = &tree.nodes[idx];

            if !node.conditions.matches(ctx) {
                return;
            }

            if let Some(action) = node.action.as_ref() {
                results.push(PolicyMatch {
                    node_id: &node.id,
                    action,
                });

                if mode == MatchMode::FirstMatch {
                    *stop = true;
                    return;
                }
            }

            for &child_idx in &node.children {
                eval_node(tree, child_idx, ctx, mode, results, stop);
                if *stop {
                    return;
                }
            }
        }

        eval_node(self, self.root, ctx, mode, &mut results, &mut stop);
        results
    }
}

/// Condition evaluation logic is tied to EvalContext.
impl ConditionSet {
    pub fn matches(&self, ctx: &EvalContext<'_>) -> bool {
        // All must match.
        for cond in &self.all {
            if !cond.matches(ctx) {
                return false;
            }
        }

        // Any: if empty, treat as "no extra requirement".
        if self.any.is_empty() {
            true
        } else {
            self.any.iter().any(|c| c.matches(ctx))
        }
    }
}

impl ConditionKind {
    pub fn matches(&self, ctx: &EvalContext<'_>) -> bool {
        match self {
            ConditionKind::Threshold { metric, min, max } => {
                let val = match ctx.metrics.get(metric) {
                    Some(v) => *v,
                    None => return false,
                };
                if let Some(lo) = min {
                    if val < *lo {
                        return false;
                    }
                }
                if let Some(hi) = max {
                    if val > *hi {
                        return false;
                    }
                }
                true
            }
            ConditionKind::TagAny { tag } => ctx.tags.contains(tag),
            ConditionKind::TagAll { tags } => {
                tags.iter().all(|t| ctx.tags.contains(t))
            }
            ConditionKind::TimeRange { start_ms, end_ms } => {
                ctx.timestamp_ms >= *start_ms && ctx.timestamp_ms <= *end_ms
            }
        }
    }
}