use std::collections::HashMap;

use crate::errors::ControllerError;
use super::EnvController;

/// Mock controller for tests/benchmarks (returns a fake pid).
pub struct MockEnvController;

impl EnvController for MockEnvController {
    fn spawn_with_env(
        _program: &str,
        _args: &[String],
        _env: &HashMap<String, String>,
        _inherit: bool,
    ) -> Result<i32, ControllerError> {
        Ok(4242) // stable fake PID for deterministic tests
    }
}