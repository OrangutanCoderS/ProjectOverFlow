use std::path::PathBuf;

/// Simple plugin identifier for now.
/// We keep it as a type alias so we can swap it later if needed.
pub type PluginId = String;

/// Static metadata we keep about each plugin.
///
/// This crate does NOT load or execute plugins. It just tracks
/// what exists and basic details needed by other crates
/// (plugin_loader, plugin_graph_builder, etc.).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginMetadata {
    /// Stable logical identifier (e.g. "netmon", "filemon.fs_watcher").
    pub id: PluginId,

    /// Human readable name (e.g. "Network Monitor").
    pub name: String,

    /// Optional semantic version string (e.g. "0.1.0").
    pub version: Option<String>,

    /// Short description for logs / UI.
    pub description: Option<String>,

    /// Optional filesystem path to the shared object / dylib / plugin bundle.
    pub library_path: Option<PathBuf>,

    /// Optional default entry symbol. Can be None if the loader
    /// uses a fixed convention.
    pub entry_symbol: Option<String>,

    /// Whether this plugin is considered enabled by configuration.
    pub enabled: bool,
}

impl PluginMetadata {
    /// Convenience constructor for the common case.
    pub fn new<S: Into<String>>(id: S, name: S) -> Self {
        let id = id.into();
        let name = name.into();
        Self {
            id,
            name,
            version: None,
            description: None,
            library_path: None,
            entry_symbol: None,
            enabled: true,
        }
    }

    /// Returns the logical key this plugin is registered under.
    pub fn key(&self) -> &PluginId {
        &self.id
    }
}