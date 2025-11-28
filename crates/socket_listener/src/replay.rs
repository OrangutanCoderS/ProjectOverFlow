use std::{
    num::NonZeroUsize,
    time::{Duration, SystemTime},
};

use chrono::{DateTime, Utc};
use lru::LruCache;
use parking_lot::Mutex;

/// Replay protection based on (origin_id, timestamp).
pub struct ReplayProtection {
    inner: Mutex<LruCache<(String, String), SystemTime>>,
    max_skew: Duration,
}

impl ReplayProtection {
    /// `capacity` - max number of entries to store
    /// `max_skew` - allowed difference between packet timestamp & now
    pub fn new(capacity: usize, max_skew: Duration) -> Self {
        // Convert to NonZeroUsize safely.
        // If someone passes 0 → fall back to a safe default of 128.
        let capacity = NonZeroUsize::new(capacity).unwrap_or_else(|| {
            NonZeroUsize::new(128).expect("128 is non-zero; qed")
        });

        Self {
            inner: Mutex::new(LruCache::new(capacity)),
            max_skew,
        }
    }

    /// Returns true if:
    ///   • timestamp is valid RFC3339
    ///   • packet isn’t older than max_skew
    ///   • packet hasn’t been seen before
    pub fn validate(&self, origin_id: &str, timestamp: &str) -> bool {
        // 1. RFC3339 parse
        let parsed: DateTime<Utc> = match timestamp.parse() {
            Ok(ts) => ts,
            Err(_) => return false,
        };

        let packet_time = parsed.into();
        let now = SystemTime::now();

        // 2. Reject future timestamps
        let diff = match now.duration_since(packet_time) {
            Ok(d) => d,
            Err(_) => return false,
        };

        // 3. Reject packets older than max_skew
        if diff > self.max_skew {
            return false;
        }

        // 4. Replay-protection via LRU
        let key = (origin_id.to_owned(), timestamp.to_owned());
        let mut cache = self.inner.lock();

        if cache.contains(&key) {
            return false; // replay
        }

        cache.put(key, packet_time);
        true
    }
}