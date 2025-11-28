use std::fs;
use std::path::{Path, PathBuf};

use tracing::debug;

use crate::error::ConfigError;
use crate::model::LinuxPortConfig;

/// Filesystem-backed loader for Linux port config YAML.
#[derive(Debug, Clone)]
pub struct LinuxPortConfigLoaderFs {
    path: PathBuf,
}

impl LinuxPortConfigLoaderFs {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Load and validate the config from disk.
    pub fn load(&self) -> Result<LinuxPortConfig, ConfigError> {
        debug!("loading LinuxPortConfig from {:?}", self.path);
        let text = fs::read_to_string(&self.path)?;
        let cfg: LinuxPortConfig = serde_yaml::from_str(&text)?;
        cfg.validate()?;
        Ok(cfg)
    }
}
