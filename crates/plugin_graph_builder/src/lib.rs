//! Plugin Graph Builder – Phase III
//!
//! Converts `PluginGraphDef` (parsed JSON/YAML) into a fully validated,
//! topologically sorted and layered graph ready for `plugin_graph_runtime`.

pub mod errors;
pub mod types;
pub mod builder;

pub use errors::PluginGraphBuilderError;
pub use types::{PluginId, PluginGraph, PluginNode};
pub use builder::PluginGraphBuilder;
