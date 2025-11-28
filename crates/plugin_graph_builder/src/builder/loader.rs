use crate::{PluginGraphBuilderError, PluginNode};
use hashbrown::HashMap;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct PluginGraphDef(pub HashMap<String, Vec<String>>);

impl PluginGraphDef {
    pub fn load_from_str(s: &str) -> Result<Self, PluginGraphBuilderError> {
        serde_json::from_str::<Self>(s)
            .map_err(|e| PluginGraphBuilderError::ParseError(e.to_string()))
    }
}
