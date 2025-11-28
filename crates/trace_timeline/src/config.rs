/// Configuration for the in-memory trace timeline.
#[derive(Debug, Clone)]
pub struct TraceTimelineConfig {
    /// Hard cap on how many events we keep in memory.
    pub max_events: usize,

    /// If `true`, inserting beyond capacity evicts the oldest events.
    /// If `false`, insertion fails with `CapacityExceeded`.
    pub evict_on_overflow: bool,

    /// Default window span in nanoseconds for replay cursors if the caller
    /// does not specify explicit boundaries.
    pub default_window_span_ns: i64,
}

impl Default for TraceTimelineConfig {
    fn default() -> Self {
        Self {
            max_events: 100_000,
            evict_on_overflow: true,
            // ~10s window by default.
            default_window_span_ns: 10_000_000_000,
        }
    }
}

impl TraceTimelineConfig {
    pub fn with_max_events(mut self, max: usize) -> Self {
        self.max_events = max;
        self
    }

    pub fn with_evict_on_overflow(mut self, evict: bool) -> Self {
        self.evict_on_overflow = evict;
        self
    }

    pub fn with_default_window_span_ns(mut self, span: i64) -> Self {
        self.default_window_span_ns = span.max(0);
        self
    }
}