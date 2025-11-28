pub mod config;
pub mod error;
pub mod model;
pub mod store;
pub mod window;

pub use crate::{
    config::TraceTimelineConfig,
    error::TraceTimelineError,
    model::{EventKind, PluginId, TimelineEvent, TimelineSource, TimelineTag},
    store::TraceTimelineStore,
    window::{TimelineCursor, TimelineWindow},
};

#[cfg(test)]
mod internal_tests {
    use super::*;

    #[test]
    fn smoke_store_compiles_and_basic_ops_work() {
        let mut store = TraceTimelineStore::new(TraceTimelineConfig::default());
        assert!(store.is_empty());

        let event = TimelineEvent::new(1_000, EventKind::MetricSample, TimelineSource::SystemMetrics);
        store.insert(event).unwrap();
        assert_eq!(store.len(), 1);

        let last = store.last_n(1);
        assert_eq!(last.len(), 1);
    }
}