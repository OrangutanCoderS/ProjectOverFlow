pub mod error;
pub mod merger;
pub mod model;

pub use crate::error::{GraphMergeError, Result};
pub use crate::merger::{merge_graphs, validate_graph};
pub use crate::model::{
    Edge, MergePolicy, MergeReport, NodeId, PluginGraph, PluginNode,
};