use std::collections::HashMap;
use std::process::Command;

use crate::errors::ControllerError;
use super::EnvController;

/// Linux-safe controller using std::process::Command for env injection.
pub struct LinuxEnvController;

impl EnvController for LinuxEnvController {
    fn spawn_with_env(
        program: &str,
        args: &[String],
        env: &HashMap<String, String>,
        inherit: bool,
    ) -> Result<i32, ControllerError> {
        let mut cmd = Command::new(program);

        if !inherit {
            cmd.env_clear();
        }
        cmd.args(args);
        cmd.envs(env.iter());

        let child = cmd.spawn().map_err(|e| ControllerError::Spawn(e.to_string()))?;
        Ok(child.id() as i32)
    }
}