use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Logical “kind” of an event in the timeline.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum EventKind {
    PluginInvocation,
    Snapshot,
    MetricSample,
    Crash,
    Annotation,
}

/// Where this event came from – helps with debugging and filtering.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TimelineSource {
    PluginEvents,
    ContextMemory,
    SystemMetrics,
    CrashLog,
    AnnotationStream,
}

pub type PluginId = String;
pub type TimelineTag = String;

/// Canonical timeline event.
///
/// Phase V requirement: this is the “memory tape” unit used by the replay
/// engine, policy evolution, and trace visualizations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEvent {
    /// Monotonic-ish timestamp in **nanoseconds** from a reference start.
    ///
    /// The caller is responsible for choosing a reference; typically the
    /// earliest timestamp seen in logs or process start time.
    pub ts_nanos: i64,

    pub kind: EventKind,
    pub source: TimelineSource,

    /// Optional plugin identifier, if applicable.
    pub plugin_id: Option<PluginId>,

    /// Free-form tags like `"anomaly"`, `"policy-change"`, `"replay-start"`.
    #[serde(default)]
    pub tags: Vec<TimelineTag>,

    /// JSON payload – plugin I/O summary, metrics, crash info, etc.
    #[serde(default)]
    pub payload: serde_json::Value,
}

impl TimelineEvent {
    /// Construct directly from a nanosecond timestamp.
    pub fn new(
        ts_nanos: i64,
        kind: EventKind,
        source: TimelineSource,
    ) -> Self {
        Self {
            ts_nanos,
            kind,
            source,
            plugin_id: None,
            tags: Vec::new(),
            payload: serde_json::Value::Null,
        }
    }

    /// Construct from a wall-clock timestamp plus a reference start.
    /// Negative deltas are clamped to 0.
    pub fn from_datetime(
        ts: DateTime<Utc>,
        ref_start: DateTime<Utc>,
        kind: EventKind,
        source: TimelineSource,
    ) -> Self {
        let delta = ts.signed_duration_since(ref_start);
        let nanos = delta.num_nanoseconds().unwrap_or(0).max(0);
        Self::new(nanos, kind, source)
    }

    pub fn with_plugin_id(mut self, id: impl Into<String>) -> Self {
        self.plugin_id = Some(id.into());
        self
    }

    pub fn with_tags<I, S>(mut self, tags: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.tags = tags.into_iter().map(Into::into).collect();
        self
    }

    pub fn with_payload(mut self, payload: serde_json::Value) -> Self {
        self.payload = payload;
        self
    }
}