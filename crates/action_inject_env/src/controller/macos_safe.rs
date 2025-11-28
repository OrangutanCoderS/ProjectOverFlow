use std::collections::HashMap;
use std::process::Command;

use crate::errors::ControllerError;
use super::EnvController;

/// macOS-safe controller using std::process::Command for env injection.
pub struct MacOSEnvController;

impl EnvController for MacOSEnvController {
    fn spawn_with_env(
        program: &str,
        args: &[String],
        env: &HashMap<String, String>,
        inherit: bool,
    ) -> Result<i32, ControllerError> {
        let mut cmd = Command::new(program);

        if !inherit {
            // Clear current process environment to get a clean surface.
            cmd.env_clear();
        }
        cmd.args(args);
        cmd.envs(env.iter());

        let child = cmd.spawn().map_err(|e| ControllerError::Spawn(e.to_string()))?;
        // Best-effort: try_id() on unix-like platforms yields pid
        let pid = child.id() as i32;
        Ok(pid)
    }
}