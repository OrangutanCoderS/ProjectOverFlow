use hashbrown::{HashMap, HashSet};
use crate::PluginId;

pub fn compute_layers(sorted: &[PluginId], adj: &HashMap<PluginId, Vec<PluginId>>)
    -> Vec<Vec<PluginId>>
{
    let mut layers = Vec::new();
    let mut assigned = HashSet::new();

    for id in sorted {
        if assigned.contains(id) {
            continue;
        }
        let mut layer = vec![id.clone()];
        assigned.insert(id.clone());

        // Place all nodes whose dependencies are fully satisfied
        for n in sorted {
            if assigned.contains(n) {
                continue;
            }
            let deps = adj.get(n).cloned().unwrap_or_default();
            if deps.iter().all(|d| assigned.contains(d)) {
                assigned.insert(n.clone());
                layer.push(n.clone());
            }
        }

        layers.push(layer);
    }

    layers
}
