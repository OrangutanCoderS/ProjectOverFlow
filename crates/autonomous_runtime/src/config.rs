use serde::{Deserialize, Serialize};

/// Configuration for the autonomous runtime loop.
///
/// This is deliberately minimal and conservative to avoid footguns.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuntimeConfig {
    /// Base tick interval in milliseconds. Must be > 0.
    pub tick_interval_ms: u64,

    /// Maximum number of events processed per tick. Must be > 0.
    pub max_events_per_tick: usize,

    /// How many ticks between "flush" operations (e.g. log flush).
    /// Must be > 0. If set to 1, flushes on every tick.
    pub flush_interval_ticks: u64,

    /// If true, any hard error in the loop will cause the runtime
    /// to transition to a failed state instead of trying to continue.
    pub fail_fast: bool,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            tick_interval_ms: 100, // 10 Hz
            max_events_per_tick: 256,
            flush_interval_ticks: 50,
            fail_fast: true,
        }
    }
}

impl RuntimeConfig {
    /// Validate that all values are in a sane range.
    ///
    /// We keep policy very strict to avoid weird degenerate states.
    pub fn validate(&self) -> Result<(), String> {
        if self.tick_interval_ms == 0 {
            return Err("tick_interval_ms must be > 0".to_string());
        }
        if self.max_events_per_tick == 0 {
            return Err("max_events_per_tick must be > 0".to_string());
        }
        if self.flush_interval_ticks == 0 {
            return Err("flush_interval_ticks must be > 0".to_string());
        }
        Ok(())
    }
}