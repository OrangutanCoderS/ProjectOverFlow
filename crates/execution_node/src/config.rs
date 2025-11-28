use std::io::Read;

use crate::error::ExecutionNodeError;
use crate::model::ExecutionConfigDef;

/// Runtime configuration type.
/// For now this is identical to the deserialized definition.
pub type ExecutionConfig = ExecutionConfigDef;

impl ExecutionConfig {
    /// Load config from a JSON reader.
    pub fn from_reader_json<R: Read>(mut r: R) -> Result<Self, ExecutionNodeError> {
        let mut buf = String::new();
        r.read_to_string(&mut buf)?;
        let cfg: ExecutionConfigDef = serde_json::from_str(&buf)?;
        Ok(cfg)
    }

    /// Load config from a YAML reader.
    pub fn from_reader_yaml<R: Read>(mut r: R) -> Result<Self, ExecutionNodeError> {
        let mut buf = String::new();
        r.read_to_string(&mut buf)?;
        let cfg: ExecutionConfigDef = serde_yaml::from_str(&buf)?;
        Ok(cfg)
    }
}
