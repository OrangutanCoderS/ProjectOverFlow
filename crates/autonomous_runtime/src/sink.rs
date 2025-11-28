use crate::error::RuntimeError;
use crate::event::RuntimeAction;

/// Abstraction over any action sink (e.g. action_* crates, logs, etc.).
///
/// Again: single-threaded, no internal concurrency assumptions.
pub trait ActionSink {
    /// Submit actions for execution. Implementations should be best-effort:
    /// one failing action should not automatically poison the entire batch
    /// unless it indicates a systemic failure.
    fn submit_actions(&mut self, actions: &[RuntimeAction]) -> Result<(), RuntimeError>;
}

/// No-op sink used for dry runs / testing.
pub struct NullActionSink;

impl ActionSink for NullActionSink {
    fn submit_actions(&mut self, _actions: &[RuntimeAction]) -> Result<(), RuntimeError> {
        Ok(())
    }
}