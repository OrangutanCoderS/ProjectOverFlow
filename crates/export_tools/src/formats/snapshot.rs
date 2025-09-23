use chrono::{DateTime, Utc};
use serde::Serialize;

/// Minimal snapshot summary for a series of records with timestamps.
#[derive(Debug, Clone, Serialize, Default, PartialEq, Eq)]
pub struct Snapshot {
    pub total: usize,
    pub first: Option<DateTime<Utc>>,
    pub last: Option<DateTime<Utc>>,
}

impl Snapshot {
    /// Update snapshot with an optional timestamp.
    pub fn update(&mut self, ts: Option<DateTime<Utc>>) {
        self.total += 1;
        if let Some(t) = ts {
            if self.first.map(|f| t < f).unwrap_or(true) {
                self.first = Some(t);
            }
            if self.last.map(|l| t > l).unwrap_or(true) {
                self.last = Some(t);
            }
        }
    }
}
