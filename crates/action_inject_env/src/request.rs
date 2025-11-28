use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Controls how injection should be performed.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum EnvInjectionMode {
    /// Validate inputs, log intent, but do not spawn or change anything.
    DryRun,
    /// Spawn a new process with injected environment variables.
    SpawnWithEnv,
}

/// Request describing what to inject and how to run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InjectRequest {
    /// Program to run (absolute path preferred).
    pub program: String,
    /// Command-line arguments (no shell expansion here).
    pub args: Vec<String>,
    /// Key/value map of environment variables to inject.
    pub env: HashMap<String, String>,
    /// If false, we clear the base environment and set only `env`.
    /// If true, we inherit current environment, then overlay `env`.
    pub inherit: bool,
    /// Mode (dry-run vs actual spawn).
    pub mode: EnvInjectionMode,
    /// Optional context for audit correlation.
    pub context: Option<String>,
}

impl Default for InjectRequest {
    fn default() -> Self {
        Self {
            program: String::new(),
            args: Vec::new(),
            env: HashMap::new(),
            inherit: true,
            mode: EnvInjectionMode::DryRun,
            context: None,
        }
    }
}