use crate::error::ExportSanitizerError;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Configuration that controls how the sanitizer behaves.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SanitizerRules {
    /// Paths we never touch (e.g. /tmp or other safe locations).
    pub whitelist_paths: Vec<String>,
    /// Keys whose values we never redact (e.g. plugin_name).
    pub whitelist_keys: Vec<String>,
    /// IPs that are allowed to pass through un-hashed (rare; use carefully).
    pub whitelist_ips: Vec<String>,
    /// If true, aggressively hash most string values (deep redact).
    pub paranoid: bool,
}

impl Default for SanitizerRules {
    fn default() -> Self {
        SanitizerRules {
            whitelist_paths: vec![
                "/tmp".to_string(),
                "/var/log".to_string(),
            ],
            whitelist_keys: vec![
                "plugin".to_string(),
                "plugin_name".to_string(),
                "plugin_id".to_string(),
            ],
            whitelist_ips: Vec::new(),
            paranoid: false,
        }
    }
}

impl SanitizerRules {
    /// Load rules from a YAML file (e.g., sanitizer_rules.yaml).
    /// Any missing fields fall back to their default values.
    pub fn from_yaml_file(path: impl AsRef<Path>) -> Result<Self, ExportSanitizerError> {
        let text = fs::read_to_string(path)?;
        Self::from_yaml_str(&text)
    }

    /// Load rules from a YAML string.
    pub fn from_yaml_str(yaml: &str) -> Result<Self, ExportSanitizerError> {
        let mut loaded: SanitizerRules = serde_yaml::from_str(yaml)?;
        // Merge with defaults: missing fields get default values.
        let default = SanitizerRules::default();

        if loaded.whitelist_paths.is_empty() {
            loaded.whitelist_paths = default.whitelist_paths;
        }
        if loaded.whitelist_keys.is_empty() {
            loaded.whitelist_keys = default.whitelist_keys;
        }
        if loaded.whitelist_ips.is_empty() {
            loaded.whitelist_ips = default.whitelist_ips;
        }

        Ok(loaded)
    }

    /// Returns true if a given key is in the whitelist.
    pub fn is_key_whitelisted(&self, key: &str) -> bool {
        self.whitelist_keys.iter().any(|k| k == key)
    }

    /// Returns true if a given path prefix is whitelisted.
    pub fn is_path_whitelisted(&self, path: &str) -> bool {
        self.whitelist_paths.iter().any(|p| path.starts_with(p))
    }

    /// Returns true if a given IP is explicitly whitelisted.
    pub fn is_ip_whitelisted(&self, ip: &str) -> bool {
        self.whitelist_ips.iter().any(|v| v == ip)
    }
}
