use std::fs;
use std::path::Path;

use hashbrown::HashSet;
use plugin_response_chain::ActionId;

use crate::engine::LogicalBranchingEngine;
use crate::error::LogicalBranchError;
use crate::model::LogicalBranchingConfig;

/// Load logical_branching.yaml from a string.
pub fn load_from_yaml_str(
    s: &str,
    allowed_actions: Option<&HashSet<ActionId>>,
) -> Result<LogicalBranchingEngine, LogicalBranchError> {
    let cfg: LogicalBranchingConfig = serde_yaml::from_str(s)?;
    LogicalBranchingEngine::from_config(cfg, allowed_actions)
}

/// Load logical_branching.yaml from a file path.
pub fn load_from_yaml_file(
    path: &Path,
    allowed_actions: Option<&HashSet<ActionId>>,
) -> Result<LogicalBranchingEngine, LogicalBranchError> {
    let data = fs::read_to_string(path)?;
    load_from_yaml_str(&data, allowed_actions)
}