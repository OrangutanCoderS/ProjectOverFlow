use crate::error::ConfigError;

/// Placeholder for future macOS-specific sensor / hook config.
/// Right now we treat macOS as "no extra config, but supported platform".
#[derive(Debug, Clone)]
pub struct MacOsHooksConfig;

impl MacOsHooksConfig {
    pub fn new_empty() -> Self {
        MacOsHooksConfig
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        Ok(())
    }
}
