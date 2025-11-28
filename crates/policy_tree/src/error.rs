use thiserror::Error;

/// Errors for building and evaluating a policy tree.
#[derive(Debug, Error)]
pub enum PolicyTreeError {
    #[error("policy tree exceeds max nodes: attempted={attempted}, max={max}")]
    NodeLimitExceeded { attempted: usize, max: usize },

    #[error("duplicate node id in policy tree: {0}")]
    DuplicateNodeId(String),

    #[error("unknown root id in policy tree: {0}")]
    UnknownRootId(String),

    #[error("unknown child id '{child_id}' referenced from node '{parent_id}'")]
    UnknownChildId { parent_id: String, child_id: String },

    #[error("cycle detected in policy tree at node '{0}'")]
    CycleDetected(String),
}