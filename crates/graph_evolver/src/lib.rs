pub mod config;
pub mod error;
pub mod model;
pub mod evolver;

pub use crate::config::EvolutionConfig;
pub use crate::error::GraphEvolverError;
pub use crate::evolver::GraphEvolver;
pub use crate::model::{
    NodeId,
    PluginNode,
    Edge,
    PluginGraph,
    TelemetrySummary,
    SuggestedEdge,
    GraphScore,
    EvolutionCandidate,
    EvolutionOp,
    EvolutionOpKind,
};
