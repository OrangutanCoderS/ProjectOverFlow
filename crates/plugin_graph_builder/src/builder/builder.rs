use hashbrown::{HashMap};
use crate::{
    PluginGraphBuilderError,
    PluginGraph,
    PluginNode,
    PluginId,
};
use super::{loader::PluginGraphDef, validator, toposort, layers};

pub struct PluginGraphBuilder;

impl PluginGraphBuilder {
    pub fn build(def: PluginGraphDef) -> Result<PluginGraph, PluginGraphBuilderError> {
        let ids: Vec<PluginId> = def.0.keys().cloned().collect();
        validator::validate_duplicates(&ids)?;

        let mut nodes = HashMap::<PluginId, PluginNode>::new();
        for (id, deps) in def.0.into_iter() {
            validator::validate_references(&deps, &ids)?;
            nodes.insert(
                id.clone(),
                PluginNode {
                    id,
                    depends_on: deps,
                },
            );
        }

        let mut adj = HashMap::<PluginId, Vec<PluginId>>::new();
        let mut rev = HashMap::<PluginId, Vec<PluginId>>::new();

        for (id, node) in &nodes {
            adj.insert(id.clone(), node.depends_on.clone());
            for dep in &node.depends_on {
                rev.entry(dep.clone()).or_default().push(id.clone());
            }
        }

        let sorted = toposort::topo_sort(&adj)?;
        let layer_groups = layers::compute_layers(&sorted, &adj);

        Ok(PluginGraph {
            nodes,
            adj,
            rev,
            layers: layer_groups,
        })
    }
}
