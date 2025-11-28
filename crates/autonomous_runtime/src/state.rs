use serde::{Deserialize, Serialize};

/// High-level state of the runtime.
#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RuntimeStatus {
    /// Created but never started.
    Initialized,
    /// Currently processing ticks.
    Running,
    /// Temporarily paused. No events are processed.
    Paused,
    /// Stopped cleanly.
    Stopped,
    /// Stopped due to a hard error.
    Failed,
}

/// Internal mutable state of the runtime.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeState {
    pub status: RuntimeStatus,
    pub tick_counter: u64,
    pub last_error: Option<String>,
}

impl RuntimeState {
    pub fn new() -> Self {
        Self {
            status: RuntimeStatus::Initialized,
            tick_counter: 0,
            last_error: None,
        }
    }

    pub fn record_error<E: std::fmt::Display>(&mut self, err: E) {
        self.last_error = Some(err.to_string());
        self.status = RuntimeStatus::Failed;
    }
}