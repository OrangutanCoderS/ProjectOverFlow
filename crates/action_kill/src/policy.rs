use serde::{Deserialize, Serialize};
use std::{fs, path::Path};
use crate::errors::ManagerError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KillPolicy {
    pub id: String,
    pub match_name: Option<String>,
    pub match_uid: Option<u32>,
    pub mode: String,
    pub cooldown_secs: Option<u64>,
    pub dry_run: bool,
    pub whitelist: Vec<String>,
    pub blacklist: Vec<String>,
}

#[derive(Debug)]
pub struct PolicyLoader;

impl PolicyLoader {
    pub fn load_from(path: &Path) -> Result<Vec<KillPolicy>, ManagerError> {
        let data = fs::read_to_string(path)?;
        let parsed: Vec<KillPolicy> = toml::from_str(&data)
            .map_err(|e| ManagerError::Config(format!("parse error: {e}")))?;
        Ok(parsed)
    }
}