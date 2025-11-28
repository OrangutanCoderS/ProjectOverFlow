use std::path::PathBuf;

use crate::error::OrchestratorError;

/// Static config for the orchestrator.
/// Later you can extend this with:
/// - policy config paths
/// - logical branching program path
/// - trigger/threshold config, etc.
#[derive(Debug, Clone)]
pub struct OrchestratorConfig {
    /// Where to search for compiled plugin dynamic libs / crates.
    pub plugin_search_paths: Vec<PathBuf>,

    /// Optional path to a graph definition (JSON/YAML) describing
    /// plugin nodes and edges.
    pub graph_config_path: Option<PathBuf>,
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            plugin_search_paths: Vec::new(),
            graph_config_path: None,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum OrchestratorState {
    Idle,
    Loaded,
    Running,
}

/// Synchronous orchestrator for the plugin graph runtime.
///
/// Right now this is deliberately minimal:
/// - it tracks lifecycle state
/// - it exposes hooks (`load`, `start`, `stop`, `reload`)
/// You can progressively inject real wiring:
/// - plugin_loader: load metadata + dynamic libs
/// - plugin_graph_builder: build PluginGraphDef
/// - plugin_graph_runtime: create and drive the runtime
pub struct PluginOrchestrator {
    config: OrchestratorConfig,
    state: OrchestratorState,
    // TODO: later
    // loader: plugin_loader::PluginLoader,
    // graph: Option<plugin_graph_runtime::PluginGraphRuntime>,
}

impl PluginOrchestrator {
    pub fn new(config: OrchestratorConfig) -> Self {
        Self {
            config,
            state: OrchestratorState::Idle,
        }
    }

    pub fn state(&self) -> OrchestratorState {
        self.state
    }

    pub fn config(&self) -> &OrchestratorConfig {
        &self.config
    }

    /// Load plugins + graph definition.
    ///
    /// Currently just flips state; you will fill this in later
    /// when you’re ready to wire all Phase III crates together.
    pub fn load(&mut self) -> Result<(), OrchestratorError> {
        // TODO:
        // 1. Use plugin_loader to scan `plugin_search_paths`
        // 2. Use plugin_graph_builder to build graph from config
        // 3. Instantiate plugin_graph_runtime with that graph
        self.state = OrchestratorState::Loaded;
        Ok(())
    }

    /// Start the plugin graph runtime loop.
    ///
    /// For now, this is synchronous and non-blocking. When you
    /// implement the actual runtime, you can decide whether
    /// `start` blocks or spawns a background thread.
    pub fn start(&mut self) -> Result<(), OrchestratorError> {
        if matches!(self.state, OrchestratorState::Idle) {
            self.load()?;
        }

        // TODO: call into PluginGraphRuntime::start()/run() when wired.
        self.state = OrchestratorState::Running;
        Ok(())
    }

    /// Stop the runtime and tear down any resources.
    pub fn stop(&mut self) -> Result<(), OrchestratorError> {
        // TODO: gracefully shut down PluginGraphRuntime instance.
        self.state = OrchestratorState::Idle;
        Ok(())
    }

    /// Full reload: stop, rebuild graph, re-start.
    pub fn reload(&mut self) -> Result<(), OrchestratorError> {
        self.stop()?;
        self.load()?;
        Ok(())
    }
}