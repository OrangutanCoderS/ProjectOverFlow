use std::path::PathBuf;

use crate::error::OrchestratorError;
use crate::orchestrator::{OrchestratorConfig, PluginOrchestrator};

/// Convenience bootstrap for Phase I/III runners.
///
/// Example future use in `daemon` crate:
/// ```ignore
/// let orchestrator = bootstrap_from_paths(
///     vec![PathBuf::from("/usr/local/overflow/plugins")],
///     Some(PathBuf::from("/etc/overflow/plugin_graph.yaml")),
/// )?;
/// orchestrator.start()?;
/// ```
pub fn bootstrap_from_paths(
    plugin_search_paths: Vec<PathBuf>,
    graph_config_path: Option<PathBuf>,
) -> Result<PluginOrchestrator, OrchestratorError> {
    let config = OrchestratorConfig {
        plugin_search_paths,
        graph_config_path,
    };

    let mut orchestrator = PluginOrchestrator::new(config);
    orchestrator.load()?;
    Ok(orchestrator)
}