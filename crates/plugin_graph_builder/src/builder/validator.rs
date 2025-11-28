use hashbrown::HashSet;
use crate::{PluginGraphBuilderError, PluginId};

pub fn validate_duplicates(ids: &[PluginId]) -> Result<(), PluginGraphBuilderError> {
    let mut seen = HashSet::new();
    for id in ids {
        if !seen.insert(id.clone()) {
            return Err(PluginGraphBuilderError::DuplicatePlugin(id.clone()));
        }
    }
    Ok(())
}

pub fn validate_references(
    ids: &[PluginId],
    all: &[PluginId],
) -> Result<(), PluginGraphBuilderError> {
    for id in ids {
        if !all.contains(id) {
            return Err(PluginGraphBuilderError::InvalidPluginRef(id.clone()));
        }
    }
    Ok(())
}
