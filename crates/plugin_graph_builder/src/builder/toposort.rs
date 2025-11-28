use hashbrown::{HashMap, HashSet};
use crate::{PluginGraphBuilderError, PluginId};

pub fn topo_sort(adj: &HashMap<PluginId, Vec<PluginId>>)
    -> Result<Vec<PluginId>, PluginGraphBuilderError>
{
    let mut indegree = HashMap::<PluginId, usize>::new();
    for (node, deps) in adj {
        indegree.entry(node.clone()).or_insert(0);
        for d in deps {
            *indegree.entry(d.clone()).or_insert(0) += 1;
        }
    }

    let mut q: Vec<PluginId> = indegree
        .iter()
        .filter(|(_, &deg)| deg == 0)
        .map(|(k, _)| k.clone())
        .collect();

    let mut out = Vec::new();

    while let Some(n) = q.pop() {
        out.push(n.clone());
        if let Some(children) = adj.get(&n) {
            for c in children {
                let e = indegree.get_mut(c).unwrap();
                *e -= 1;
                if *e == 0 {
                    q.push(c.clone());
                }
            }
        }
    }

    if out.len() != adj.len() {
        Err(PluginGraphBuilderError::CycleDetected)
    } else {
        Ok(out)
    }
}
