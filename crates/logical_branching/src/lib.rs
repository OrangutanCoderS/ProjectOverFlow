//! logical_branching – Phase III global policy override / firewall.
//!
//! This crate evaluates `logical_branching.yaml` against:
//! - current context flags
//! - plugin runtime states
//! - system modifiers (secure_mode, battery, etc.)
//! and filters candidate actions coming from `plugin_response_chain`.

mod error;
mod model;
mod engine;
mod loader;

pub use error::LogicalBranchError;
pub use engine::LogicalBranchingEngine;
pub use loader::{load_from_yaml_file, load_from_yaml_str};
pub use model::{
    BranchDecision, LogicalBranchingConfig, LogicalRule, PluginRuntimeState, RuntimeSnapshot,
};

// Re-export ActionId so callers don't need to pull plugin_response_chain directly.
pub use plugin_response_chain::ActionId;