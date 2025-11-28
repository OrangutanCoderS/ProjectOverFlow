use crate::errors::PluginGraphError;
use crate::graph_def::PluginGraphDef;
use crate::types::{PluginId, PluginIndex};
use hashbrown::{HashMap, HashSet};

pub struct PluginGraphRuntime {
    pub index: HashMap<PluginId, PluginIndex>,
    pub reverse_index: Vec<PluginId>,
    pub outgoing: Vec<Vec<PluginIndex>>,
}

impl PluginGraphRuntime {
    pub fn load(def: PluginGraphDef) -> Result<Self, PluginGraphError> {
        let mut index = HashMap::new();
        let mut reverse_index = Vec::new();

        // Assign stable numeric IDs
        for plugin in def.edges.keys() {
            if index.contains_key(plugin) {
                return Err(PluginGraphError::DuplicatePlugin(plugin.clone()));
            }
            let id = reverse_index.len() as u16;
            index.insert(plugin.clone(), id);
            reverse_index.push(plugin.clone());
        }

        // Build adjacency
        let mut outgoing = vec![Vec::new(); reverse_index.len()];

        for (from, tos) in &def.edges {
            let from_id = *index.get(from).ok_or_else(|| PluginGraphError::UnknownPlugin(from.clone()))?;

            for to in tos {
                let to_id = *index.get(to).ok_or_else(|| PluginGraphError::UnknownPlugin(to.clone()))?;
                outgoing[from_id as usize].push(to_id);
            }
        }

        // Validate DAG
        let rt = PluginGraphRuntime { index, reverse_index, outgoing };
        rt.check_for_cycles()?;

        Ok(rt)
    }

    fn check_for_cycles(&self) -> Result<(), PluginGraphError> {
        let mut visited = vec![false; self.outgoing.len()];
        let mut stack = vec![false; self.outgoing.len()];

        for i in 0..self.outgoing.len() {
            if !visited[i] && self.dfs(i, &mut visited, &mut stack) {
                return Err(PluginGraphError::CycleDetected);
            }
        }
        Ok(())
    }

    fn dfs(&self, node: usize, visited: &mut [bool], stack: &mut [bool]) -> bool {
        visited[node] = true;
        stack[node] = true;

        for &next in &self.outgoing[node] {
            let next_us = next as usize;

            if !visited[next_us] && self.dfs(next_us, visited, stack) {
                return true;
            } else if stack[next_us] {
                return true;
            }
        }

        stack[node] = false;
        false
    }
}