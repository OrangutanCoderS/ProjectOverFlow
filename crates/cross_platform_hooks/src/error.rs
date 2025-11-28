use std::io;

use thiserror::Error;

use crate::model::PlatformKind;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("io error: {0}")]
    Io(#[from] io::Error),

    #[error("yaml error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("unsupported platform: {0:?}")]
    Unsupported(PlatformKind),

    #[error("invalid config: {0}")]
    Invalid(String),
}
