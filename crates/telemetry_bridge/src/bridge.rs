use crate::error::TelemetryError;
use crate::packet::TelemetryPacket;
use crate::sink::TelemetrySink;
use parking_lot::{Mutex, RwLock};
use std::collections::VecDeque;
use std::sync::Arc;

/// Configuration for TelemetryBridge behavior.
#[derive(Debug, Clone)]
pub struct BridgeConfig {
    /// Enable in-memory ring buffer of recent packets (0 = disabled).
    pub cache_capacity: usize,
}

impl Default for BridgeConfig {
    fn default() -> Self {
        Self { cache_capacity: 256 }
    }
}

/// The bridge: accept packets, fan-out to sinks, optionally keep an in-memory cache.
pub struct TelemetryBridge {
    sinks: RwLock<Vec<Arc<dyn TelemetrySink>>>,
    cache: Option<Mutex<VecDeque<TelemetryPacket>>>,
    cfg: BridgeConfig,
}

impl TelemetryBridge {
    pub fn new(cfg: BridgeConfig) -> Self {
        let cache = if cfg.cache_capacity == 0 {
            None
        } else {
            Some(Mutex::new(VecDeque::with_capacity(cfg.cache_capacity)))
        };
        Self {
            sinks: RwLock::new(Vec::new()),
            cache,
            cfg,
        }
    }

    /// Register a sink. Safe to call from multiple threads before/after record calls.
    pub fn add_sink(&self, sink: Arc<dyn TelemetrySink>) {
        let mut sinks = self.sinks.write();
        sinks.push(sink);
    }

    /// Record a packet. On failure of any sink, returns the first error.
    /// We do not partially rollback other sinks; failures are fail-fast to surface issues.
    pub fn record(&self, packet: TelemetryPacket) -> Result<(), TelemetryError> {
        // Write to sinks first (I/O), then update cache (cheap).
        // Clone only for cache if needed; otherwise we move the packet once.
        {
            let sinks = self.sinks.read();
            for s in sinks.iter() {
                s.publish(&packet)?; // fail fast
            }
        }

        if let Some(cache) = &self.cache {
            let mut q = cache.lock();
            if q.len() == self.cfg.cache_capacity {
                q.pop_front();
            }
            q.push_back(packet);
        }

        Ok(())
    }

    /// Fetch up to `n` most recent packets from cache (most recent last).
    /// Returns empty vec if cache disabled.
    pub fn latest(&self, n: usize) -> Vec<TelemetryPacket> {
        if let Some(cache) = &self.cache {
            let q = cache.lock();
            let k = n.min(q.len());
            q.iter().skip(q.len() - k).cloned().collect()
        } else {
            Vec::new()
        }
    }

    /// Return the number of registered sinks.
    pub fn sinks_len(&self) -> usize {
        self.sinks.read().len()
    }
}
