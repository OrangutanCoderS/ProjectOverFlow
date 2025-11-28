use crate::{store::TraceTimelineStore, model::TimelineEvent};

/// A view over a contiguous slice of events.
#[derive(Debug, Clone, Copy)]
pub struct TimelineWindow<'a> {
    events: &'a [TimelineEvent],
}

impl<'a> TimelineWindow<'a> {
    pub fn new(events: &'a [TimelineEvent]) -> Self {
        Self { events }
    }

    pub fn events(&self) -> &'a [TimelineEvent] {
        self.events
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }
}

/// Sliding cursor over the store for replay-style consumption.
#[derive(Debug)]
pub struct TimelineCursor<'a> {
    store: &'a TraceTimelineStore,
    pos: usize,
}

impl<'a> TimelineCursor<'a> {
    pub fn new(store: &'a TraceTimelineStore) -> Self {
        Self { store, pos: 0 }
    }

    /// Return the next `count` events as a window.
    /// Returns `None` when we reach the end.
    pub fn next_by_count(&mut self, count: usize) -> Option<TimelineWindow<'a>> {
        if count == 0 || self.pos >= self.store.len() {
            return None;
        }

        let len = self.store.len();
        let end = (self.pos + count).min(len);
        let window = TimelineWindow::new(&self.store.events()[self.pos..end]);
        self.pos = end;
        Some(window)
    }

    /// Reset the cursor back to the beginning.
    pub fn reset(&mut self) {
        self.pos = 0;
    }

    /// Are we at the end?
    pub fn is_finished(&self) -> bool {
        self.pos >= self.store.len()
    }

    /// Current position (index into the store).
    pub fn position(&self) -> usize {
        self.pos
    }
}