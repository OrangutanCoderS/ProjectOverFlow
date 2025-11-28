use hashbrown::{HashMap, HashSet};

pub type PluginId = String;

#[derive(Debug, Clone)]
pub struct PluginNode {
    pub id: PluginId,
    pub depends_on: Vec<PluginId>,
}

#[derive(Debug, Clone)]
pub struct PluginGraph {
    pub nodes: HashMap<PluginId, PluginNode>,
    pub adj: HashMap<PluginId, Vec<PluginId>>,
    pub rev: HashMap<PluginId, Vec<PluginId>>,
    pub layers: Vec<Vec<PluginId>>,
}
