use crate::error::ExecError;
use crate::model::{ExecutionInput, ExecutionStep};

/// Abstract backend that actually performs actions.
///
/// This crate does NOT depend on any OverFlow action crates.
/// Instead, higher layers provide an implementation that delegates to them.
pub trait ActionBackend {
    fn execute_step(
        &mut self,
        step: &ExecutionStep,
        input: &ExecutionInput,
    ) -> Result<(), ExecError>;
}

/// A backend that does nothing but report success.
/// Used for tests and micro-benchmarks.
#[derive(Debug, Default)]
pub struct NullBackend;

impl ActionBackend for NullBackend {
    fn execute_step(
        &mut self,
        _step: &ExecutionStep,
        _input: &ExecutionInput,
    ) -> Result<(), ExecError> {
        Ok(())
    }
}
