use thiserror::Error;
use crate::model::PluginId;

#[derive(Debug, Error)]
pub enum PluginRegistryError {
    #[error("Plugin `{0}` is already registered")]
    Duplicate(PluginId),

    #[error("Plugin `{0}` is not registered")]
    NotFound(PluginId),
}