use std::collections::HashMap;

use crate::errors::ControllerError;

/// Abstract controller to prepare and spawn processes with injected env.
pub trait EnvController {
    /// Spawn a child process, injecting environment according to parameters.
    /// Returns the spawned child's PID on success.
    fn spawn_with_env(
        program: &str,
        args: &[String],
        env: &HashMap<String, String>,
        inherit: bool,
    ) -> Result<i32, ControllerError>;
}

#[cfg(target_os = "macos")]
pub mod macos_safe;
#[cfg(target_os = "linux")]
pub mod linux_safe;

// Always available for tests and benches.
pub mod mock;