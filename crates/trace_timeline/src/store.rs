use std::cmp::Ordering;

use crate::{
    config::TraceTimelineConfig,
    error::TraceTimelineError,
    model::TimelineEvent,
};

/// In-memory sorted timeline store.
///
/// Internally backed by a Vec sorted by `ts_nanos`.
#[derive(Debug)]
pub struct TraceTimelineStore {
    config: TraceTimelineConfig,
    events: Vec<TimelineEvent>,
}

impl TraceTimelineStore {
    pub fn new(config: TraceTimelineConfig) -> Self {
        Self {
            config,
            events: Vec::new(),
        }
    }

    /// Construct from an unsorted iterator of events.
    /// Events with negative timestamps are dropped.
    pub fn from_events<I>(config: TraceTimelineConfig, iter: I) -> Self
    where
        I: IntoIterator<Item = TimelineEvent>,
    {
        let mut events: Vec<_> = iter
            .into_iter()
            .filter(|e| e.ts_nanos >= 0)
            .collect();

        events.sort_by_key(|e| e.ts_nanos);
        if events.len() > config.max_events {
            let start = events.len() - config.max_events;
            events.drain(..start);
        }

        Self { config, events }
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    pub fn config(&self) -> &TraceTimelineConfig {
        &self.config
    }

    pub fn events(&self) -> &[TimelineEvent] {
        &self.events
    }

    /// Insert a single event, keeping the store sorted.
    ///
    /// Capacity logic:
    /// - if `evict_on_overflow == false` and full => `CapacityExceeded`
    /// - if `evict_on_overflow == true` and full => drop oldest, then insert.
    pub fn insert(&mut self, event: TimelineEvent) -> Result<(), TraceTimelineError> {
        if event.ts_nanos < 0 {
            return Err(TraceTimelineError::InvalidTimestamp(format!(
                "negative ts_nanos: {}",
                event.ts_nanos
            )));
        }

        if self.events.len() == self.config.max_events {
            if self.config.evict_on_overflow {
                // Drop oldest event (store is sorted by timestamp).
                self.events.remove(0);
            } else {
                return Err(TraceTimelineError::CapacityExceeded(
                    self.config.max_events,
                ));
            }
        }

        let idx = match self
            .events
            .binary_search_by_key(&event.ts_nanos, |e| e.ts_nanos)
        {
            Ok(i) | Err(i) => i,
        };

        self.events.insert(idx, event);
        Ok(())
    }

    /// Bulk insert; stops at the first error.
    pub fn insert_many<I>(&mut self, events: I) -> Result<(), TraceTimelineError>
    where
        I: IntoIterator<Item = TimelineEvent>,
    {
        for e in events {
            self.insert(e)?;
        }
        Ok(())
    }

    /// Get a window in [start_ns, end_ns).
    ///
    /// Returns a subslice of the underlying Vec; empty slice if no overlap.
    pub fn window_by_range(&self, start_ns: i64, end_ns: i64) -> &[TimelineEvent] {
        if self.events.is_empty() || start_ns >= end_ns {
            return &self.events[0..0];
        }

        let start_idx = lower_bound(&self.events, start_ns);
        let end_idx = upper_bound(&self.events, end_ns);

        if start_idx >= end_idx {
            &self.events[0..0]
        } else {
            &self.events[start_idx..end_idx]
        }
    }

    /// Get the last `n` events; fewer if the store is smaller.
    pub fn last_n(&self, n: usize) -> &[TimelineEvent] {
        if self.events.is_empty() || n == 0 {
            return &self.events[0..0];
        }

        let len = self.events.len();
        let start = len.saturating_sub(n);
        &self.events[start..len]
    }

    /// Get events within `[center - radius_ns, center + radius_ns]`.
    pub fn around(&self, center_ns: i64, radius_ns: i64) -> &[TimelineEvent] {
        if self.events.is_empty() || radius_ns < 0 {
            return &self.events[0..0];
        }

        let start = center_ns.saturating_sub(radius_ns);
        let end = center_ns.saturating_add(radius_ns);

        self.window_by_range(start, end + 1)
    }

    /// Export full timeline as a JSON array string.
    pub fn to_json_string(&self) -> Result<String, TraceTimelineError> {
        Ok(serde_json::to_string(&self.events)?)
    }

    /// Load timeline from a JSON array string.
    pub fn from_json_str(
        config: TraceTimelineConfig,
        s: &str,
    ) -> Result<Self, TraceTimelineError> {
        let events: Vec<TimelineEvent> = serde_json::from_str(s)?;
        Ok(Self::from_events(config, events))
    }
}

/// Lower-bound binary search on ts_nanos.
fn lower_bound(events: &[TimelineEvent], ts_ns: i64) -> usize {
    let mut low = 0;
    let mut high = events.len();

    while low < high {
        let mid = (low + high) / 2;
        match events[mid].ts_nanos.cmp(&ts_ns) {
            Ordering::Less => low = mid + 1,
            _ => high = mid,
        }
    }

    low
}

/// Upper-bound binary search on ts_nanos.
fn upper_bound(events: &[TimelineEvent], ts_ns: i64) -> usize {
    let mut low = 0;
    let mut high = events.len();

    while low < high {
        let mid = (low + high) / 2;
        match events[mid].ts_nanos.cmp(&ts_ns) {
            Ordering::Greater => high = mid,
            _ => low = mid + 1,
        }
    }

    low
}