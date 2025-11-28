use context_graph::graph::ContextGraph;
use execution_node::ExecutionConfig;
use plugin_response_chain::PluginResponseChain;

/// ChainExecutor – Phase III glue between:
/// - ContextGraph (which plugins / nodes are reachable),
/// - PluginResponseChain (what actions each plugin/severity/pattern implies),
/// - ExecutionConfig (how execution nodes are configured).
///
/// In Phase III this is intentionally thin: it just bundles the three
/// subsystems together so that higher layers (e.g. autonomous_runtime)
/// can build and run `ExecutionPlan`s as needed.
///
/// Phase IV / V can extend this to actually materialize and execute
/// `ExecutionPlan`s per plugin, but we do **not** assume any concrete
/// constructors or methods here to keep the coupling minimal and
/// compilation stable.
#[derive(Debug)]
pub struct ChainExecutor<'a> {
    /// Static trigger graph built from `context_graph` crate.
    pub graph: &'a ContextGraph,

    /// Plugin → severity → pattern → [action_id] mapping.
    pub chains: &'a PluginResponseChain,

    /// Shared execution configuration for downstream execution plans.
    pub exec_config: &'a ExecutionConfig,
}

impl<'a> ChainExecutor<'a> {
    /// Create a new ChainExecutor over existing subsystems.
    pub fn new(
        graph: &'a ContextGraph,
        chains: &'a PluginResponseChain,
        exec_config: &'a ExecutionConfig,
    ) -> Self {
        Self {
            graph,
            chains,
            exec_config,
        }
    }

    /// Placeholder for future hot-path helpers:
    /// e.g. resolve actions for a given (plugin, severity, pattern).
    ///
    /// Keeping this minimal for now to avoid guessing at higher-level API
    /// shapes – the important part for Phase III is that the types line up
    /// and are accessible from a single place.
    pub fn chains(&self) -> &PluginResponseChain {
        self.chains
    }

    pub fn graph(&self) -> &ContextGraph {
        self.graph
    }

    pub fn exec_config(&self) -> &ExecutionConfig {
        self.exec_config
    }
}