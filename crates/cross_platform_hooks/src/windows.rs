use std::fs;
use std::path::{Path, PathBuf};

use tracing::debug;

use crate::error::ConfigError;
use crate::model::WindowsHooksConfig;

/// Filesystem-backed loader for Windows hooks config YAML.
/// Note: pure-data loader; no direct ETW/WMI bindings here.
#[derive(Debug, Clone)]
pub struct WindowsHooksLoaderFs {
    path: PathBuf,
}

impl WindowsHooksLoaderFs {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn load(&self) -> Result<WindowsHooksConfig, ConfigError> {
        debug!("loading WindowsHooksConfig from {:?}", self.path);
        let text = fs::read_to_string(&self.path)?;
        let cfg: WindowsHooksConfig = serde_yaml::from_str(&text)?;
        cfg.validate()?;
        Ok(cfg)
    }
}
