use crate::types::PluginId;
use serde::{Serialize, Deserialize};
use hashbrown::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginGraphDef {
    /// For each plugin, the list of downstream plugins called next.
    pub edges: HashMap<PluginId, Vec<PluginId>>,
}