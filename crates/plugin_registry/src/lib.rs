pub mod model;
pub mod error;
pub mod registry;

pub use crate::error::PluginRegistryError;
pub use crate::model::{PluginId, PluginMetadata};
pub use crate::registry::PluginRegistry;