use serde::{Deserialize, Serialize};

use crate::error::ConfigError;

/// Supported platforms as seen by OverFlow.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatformKind {
    Linux,
    Windows,
    MacOs,
    Unknown,
}

impl PlatformKind {
    /// Detect current platform using compile-time constants.
    pub fn detect() -> Self {
        match std::env::consts::OS {
            "linux" => PlatformKind::Linux,
            "windows" => PlatformKind::Windows,
            "macos" => PlatformKind::MacOs,
            _ => PlatformKind::Unknown,
        }
    }
}

/// Default policy when no explicit rule matches.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DefaultPolicy {
    Allow,
    Deny,
}

/// Network protocol.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Protocol {
    Tcp,
    Udp,
    TcpUdp,
}

/// A single Linux port rule.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LinuxPortRule {
    pub port: u16,
    pub protocol: Protocol,
    /// Logical tag / service name, e.g. "overflow-daemon"
    pub tag: String,
    /// Whether this port is considered allowed/legit for that tag.
    pub allow: bool,
    #[serde(default)]
    pub note: Option<String>,
}

/// Linux port config: a list of rules + default fallback.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LinuxPortConfig {
    pub rules: Vec<LinuxPortRule>,
    pub default_policy: DefaultPolicy,
}

impl LinuxPortConfig {
    /// Basic structural validation.
    pub fn validate(&self) -> Result<(), ConfigError> {
        for rule in &self.rules {
            if rule.port == 0 {
                return Err(ConfigError::Invalid(format!(
                    "rule for tag '{}' has invalid port 0",
                    rule.tag
                )));
            }
        }
        Ok(())
    }
}

/// One ETW provider or similar Windows event source.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WindowsProviderConfig {
    pub name: String,
    #[serde(default)]
    pub guid: Option<String>,
    /// Logging level (e.g., "Informational", "Warning", "Verbose").
    pub level: String,
    #[serde(default)]
    pub keywords: Vec<String>,
}

/// Windows hooks config: ETW providers + WMI queries.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WindowsHooksConfig {
    #[serde(default)]
    pub providers: Vec<WindowsProviderConfig>,
    /// WMI queries to poll for additional telemetry.
    #[serde(default)]
    pub wmi_queries: Vec<String>,
}

impl WindowsHooksConfig {
    pub fn validate(&self) -> Result<(), ConfigError> {
        for p in &self.providers {
            if p.name.trim().is_empty() {
                return Err(ConfigError::Invalid(
                    "provider name must not be empty".to_string(),
                ));
            }
        }
        Ok(())
    }
}
