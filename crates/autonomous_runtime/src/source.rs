use crate::error::RuntimeError;
use crate::event::ModuleEvent;

/// Abstraction over any event-producing subsystem.
///
/// Single-threaded: implementors are expected to be internally safe
/// for repeated calls from the same thread only.
pub trait EventSource {
    /// Poll up to `max_events` new events.
    ///
    /// Implementations should never block for long – they should either:
    /// - return immediately with whatever is available, or
    /// - do a short non-busy wait bounded by the tick interval.
    fn poll_events(&mut self, max_events: usize) -> Result<Vec<ModuleEvent>, RuntimeError>;
}

/// Simple no-op source used in tests or when running with no inputs.
pub struct NullEventSource;

impl EventSource for NullEventSource {
    fn poll_events(&mut self, _max_events: usize) -> Result<Vec<ModuleEvent>, RuntimeError> {
        Ok(Vec::new())
    }
}