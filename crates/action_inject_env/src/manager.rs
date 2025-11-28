use std::collections::HashMap;

use crate::audit::write_audit_log;
use crate::errors::{ControllerError, ManagerError};
use crate::request::{EnvInjectionMode, InjectRequest};

use crate::controller::EnvController;

#[cfg(target_os = "macos")]
use crate::controller::macos_safe::MacOSEnvController as PlatformCtrl;

#[cfg(target_os = "linux")]
use crate::controller::linux_safe::LinuxEnvController as PlatformCtrl;

/// Security/robustness caps to prevent pathological inputs.
const MAX_ENV_VARS: usize = 256;
const MAX_ENV_TOTAL_BYTES: usize = 64 * 1024; // 64 KiB
const MAX_KEY_LEN: usize = 256;
const MAX_VAL_LEN: usize = 8 * 1024;

pub struct ActionInjectEnvManager;

impl Default for ActionInjectEnvManager {
    fn default() -> Self {
        Self
    }
}

impl ActionInjectEnvManager {
    /// Validate request content and compute simple metrics.
    fn validate(req: &InjectRequest) -> Result<(), ManagerError> {
        if req.program.trim().is_empty() {
            return Err(ManagerError::Validation("program is empty".into()));
        }

        if req.env.len() > MAX_ENV_VARS {
            return Err(ManagerError::Validation(format!(
                "too many env vars: {} > {}",
                req.env.len(),
                MAX_ENV_VARS
            )));
        }

        let mut total = 0usize;
        for (k, v) in &req.env {
            if k.is_empty() {
                return Err(ManagerError::Validation("env key is empty".into()));
            }
            if k.len() > MAX_KEY_LEN {
                return Err(ManagerError::Validation(format!(
                    "env key too long: {} > {}",
                    k.len(),
                    MAX_KEY_LEN
                )));
            }
            if k.bytes().any(|b| b == 0 || b == b'=' || b == b'\n') {
                return Err(ManagerError::Validation(format!(
                    "env key contains forbidden bytes: {k:?}"
                )));
            }
            if v.len() > MAX_VAL_LEN {
                return Err(ManagerError::Validation(format!(
                    "env value too long for key `{k}`: {} > {}",
                    v.len(),
                    MAX_VAL_LEN
                )));
            }
            if v.bytes().any(|b| b == 0) {
                return Err(ManagerError::Validation(format!(
                    "env value contains NUL for key `{k}`"
                )));
            }
            total = total.saturating_add(k.len() + 1 + v.len()); // key='=' value
            if total > MAX_ENV_TOTAL_BYTES {
                return Err(ManagerError::Validation(format!(
                    "total env size exceeds {} bytes",
                    MAX_ENV_TOTAL_BYTES
                )));
            }
        }

        // On both macOS and Linux we only support pre-launch injection safely.
        Ok(())
    }

    /// Execute the request. Returns Ok(Some(pid)) on SpawnWithEnv; Ok(None) for DryRun.
    pub fn execute(&self, req: InjectRequest) -> Result<Option<i32>, ManagerError> {
        Self::validate(&req)?;

        // Redact values in audit; store keys only.
        let env_keys_iter = req.env.keys().map(|s| s.as_str());
        let ctx = req.context.as_deref().unwrap_or("no_context");
        match req.mode {
            EnvInjectionMode::DryRun => {
                write_audit_log(&req.program, None, "DryRun", env_keys_iter, req.inherit, ctx)
                    .map_err(ManagerError::Audit)?;
                Ok(None)
            }
            EnvInjectionMode::SpawnWithEnv => {
                // Shallow clone is fine; size was validated.
                let env_map: HashMap<String, String> = req.env.clone();
                let pid = PlatformCtrl::spawn_with_env(&req.program, &req.args, &env_map, req.inherit)
                    .map_err(|e| ManagerError::Spawn(e.to_string()))?;

                // For audit we log the PID.
                let env_keys_iter = env_map.keys().map(|s| s.as_str());
                write_audit_log(&req.program, Some(pid), "SpawnWithEnv", env_keys_iter, req.inherit, ctx)
                    .map_err(ManagerError::Audit)?;
                Ok(Some(pid))
            }
        }
    }
}