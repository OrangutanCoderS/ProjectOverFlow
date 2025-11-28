use hashbrown::HashMap;

use crate::error::PluginRegistryError;
use crate::model::{PluginId, PluginMetadata};

/// In-memory registry of plugins.
///
/// This is deliberately dumb and synchronous:
/// - No loading
/// - No dynamic linkage
/// - No IO
///
/// Other crates are free to wrap this in async / IPC / whatever.
#[derive(Debug, Default)]
pub struct PluginRegistry {
    plugins: HashMap<PluginId, PluginMetadata>,
}

impl PluginRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }

    /// Number of registered plugins.
    pub fn len(&self) -> usize {
        self.plugins.len()
    }

    pub fn is_empty(&self) -> bool {
        self.plugins.is_empty()
    }

    /// Insert metadata for a plugin.
    ///
    /// Fails if the id is already present — we don't silently override.
    pub fn register(&mut self, meta: PluginMetadata) -> Result<(), PluginRegistryError> {
        let id = meta.id.clone();
        if self.plugins.contains_key(&id) {
            return Err(PluginRegistryError::Duplicate(id));
        }
        self.plugins.insert(id, meta);
        Ok(())
    }

    /// Remove a plugin from the registry and return its metadata.
    pub fn unregister<S: AsRef<str>>(
        &mut self,
        id: S,
    ) -> Result<PluginMetadata, PluginRegistryError> {
        let key = id.as_ref().to_string();
        self.plugins
            .remove(&key)
            .ok_or(PluginRegistryError::NotFound(key))
    }

    /// Get read-only access to a plugin's metadata.
    pub fn get<S: AsRef<str>>(&self, id: S) -> Option<&PluginMetadata> {
        self.plugins.get(id.as_ref())
    }

    /// Get mutable access to a plugin's metadata (for toggling `enabled`, etc.).
    pub fn get_mut<S: AsRef<str>>(&mut self, id: S) -> Option<&mut PluginMetadata> {
        self.plugins.get_mut(id.as_ref())
    }

    /// Returns true if a plugin with this id is registered.
    pub fn is_registered<S: AsRef<str>>(&self, id: S) -> bool {
        self.plugins.contains_key(id.as_ref())
    }

    /// Iterate over all registered plugins.
    pub fn iter(&self) -> impl Iterator<Item = (&PluginId, &PluginMetadata)> {
        self.plugins.iter()
    }

    /// Clear the registry.
    pub fn clear(&mut self) {
        self.plugins.clear();
    }
}