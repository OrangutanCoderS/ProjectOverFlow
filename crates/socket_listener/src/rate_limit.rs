use std::{
    collections::HashMap,
    net::IpAddr,
    time::{Duration, Instant},
};

use parking_lot::Mutex;

/// Simple sliding-window rate limiter keyed by peer IP.
///
/// Not cryptographically strong; it is operational protection against
/// accidental or naive abuse.
pub struct RateLimiter {
    inner: Mutex<HashMap<IpAddr, Vec<Instant>>>,
    window: Duration,
    max_events: usize,
}

impl RateLimiter {
    /// Create a new rate limiter.
    ///
    /// * `window` - time window to consider (e.g., 1 second)
    /// * `max_events` - max allowed events per IP in the window
    pub fn new(window: Duration, max_events: usize) -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
            window,
            max_events,
        }
    }

    /// Returns true if this IP is currently over limit.
    pub fn is_limited(&self, ip: &IpAddr) -> bool {
        let now = Instant::now();
        let mut map = self.inner.lock();

        let entry = map.entry(*ip).or_insert_with(Vec::new);

        // Drop events older than the window
        let cutoff = now - self.window;
        entry.retain(|t| *t >= cutoff);

        if entry.len() >= self.max_events {
            return true;
        }

        entry.push(now);
        false
    }
}
