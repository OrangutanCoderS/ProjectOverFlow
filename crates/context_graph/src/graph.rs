use serde::{Deserialize, Serialize};
use hashbrown::HashMap;

use crate::{
    edge::{Edge, EdgeDef},
    errors::ContextGraphError,
    node::{Node, NodeKey, NodeMeta},
};

const MAX_NODES: usize = 8192;

/// Full graph definition loaded from JSON/YAML.
/// Must derive both Serialize + Deserialize for round-trip tests.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphDef {
    pub nodes: Vec<NodeDef>,
    pub edges: HashMap<String, Vec<EdgeDef>>, // Fully serde-compatible
}

/// Node definition used in config.
/// MUST derive Serialize + Deserialize or serde fails.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeDef {
    pub key: NodeKey,
    pub meta: NodeMeta,
}

/// Internal graph representation.
#[derive(Debug)]
pub struct ContextGraph {
    pub nodes: Vec<Node>,
    pub outgoing: Vec<Vec<Edge>>,
    pub incoming: Vec<Vec<u16>>,
    pub index: HashMap<NodeKey, u16>,
}

impl ContextGraph {
    /// Load a validated, static graph from a configuration definition.
    pub fn load(def: GraphDef) -> Result<Self, ContextGraphError> {
        // --------------------------------------------------------------
        // 1. Node limit check
        // --------------------------------------------------------------
        if def.nodes.len() > MAX_NODES {
            return Err(ContextGraphError::NodeLimitExceeded {
                attempted: def.nodes.len(),
                max: MAX_NODES,
            });
        }

        // --------------------------------------------------------------
        // 2. Preallocate internal structures
        // --------------------------------------------------------------
        let mut nodes = Vec::with_capacity(def.nodes.len());
        let mut outgoing = vec![Vec::<Edge>::new(); def.nodes.len()];
        let mut incoming = vec![Vec::<u16>::new(); def.nodes.len()];
        let mut index: HashMap<NodeKey, u16> = HashMap::new();

        // --------------------------------------------------------------
        // 3. Register nodes & detect duplicates
        // --------------------------------------------------------------
        for (i, ndef) in def.nodes.into_iter().enumerate() {
            let id = i as u16;

            if index.contains_key(&ndef.key) {
                return Err(ContextGraphError::DuplicateNodeKey);
            }

            index.insert(ndef.key.clone(), id);
            nodes.push(Node {
                id,
                key: ndef.key,
                meta: ndef.meta,
            });
        }

        // --------------------------------------------------------------
        // 4. Build edges
        // --------------------------------------------------------------
        for (from_key_str, edge_list) in def.edges.into_iter() {
            // Resolve node key based on its actual variant
            let from_key = Self::resolve_key(&index, &from_key_str)
                .ok_or(ContextGraphError::UnknownNodeKey)?;

            let from_id = index[&from_key];

            // Process edge list
            for e in edge_list {
                // 4a. Destination range check
                if (e.to as usize) >= nodes.len() {
                    return Err(ContextGraphError::UnknownNodeKey);
                }

                // 4b. Construct edge (auto-default conditions)
                let edge = Edge {
                    to: e.to,
                    weight: e.weight,
                    label: e.label.clone(),
                    conditions: e
                        .conditions
                        .clone()
                        .unwrap_or_else(|| Default::default()),
                };

                // 4c. Validate weight or structural constraints
                edge.validate()
                    .map_err(|w| ContextGraphError::InvalidWeight(w))?;

                outgoing[from_id as usize].push(edge);
                incoming[e.to as usize].push(from_id);
            }
        }

        // --------------------------------------------------------------
        // 5. Return fully constructed runtime graph
        // --------------------------------------------------------------
        Ok(Self {
            nodes,
            outgoing,
            incoming,
            index,
        })
    }

    /// Resolve a string into the actual NodeKey variant.
    /// We search all known keys and match the inner string.
    fn resolve_key(index: &HashMap<NodeKey, u16>, raw: &str) -> Option<NodeKey> {
    for key in index.keys() {
        match key {
            NodeKey::Process(p) if p == raw => return Some(key.clone()),
            NodeKey::FilePath(p) if p == raw => return Some(key.clone()),
            NodeKey::Custom(p) if p == raw => return Some(key.clone()),
            _ => {},
        }
    }
    None
}
}