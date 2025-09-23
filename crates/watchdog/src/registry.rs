use chrono::Utc; // DateTime unused, removed
use parking_lot::RwLock;
use std::{
    collections::HashMap,
    sync::Arc,
    time::Duration,
};
use once_cell::sync::Lazy; // <-- FIX: bring in once_cell

use crate::error::WatchdogError;

/// Identifier alias for monitored entries.
pub type UnitId = String;

/// What kind of unit is supervised.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnitKind {
    ThreadFn,
    Process,
}

/// Restart policy with exponential backoff.
#[derive(Debug, Clone)]
pub struct RestartPolicy {
    pub max_restarts: u32,
    pub base_backoff: Duration,
    pub max_backoff: Duration,
}

impl Default for RestartPolicy {
    fn default() -> Self {
        Self {
            max_restarts: 5,
            base_backoff: Duration::from_millis(250),
            max_backoff: Duration::from_secs(10),
        }
    }
}

/// Register specification. `restart_cb` is invoked on restart.
#[derive(Clone)]
pub struct RegisterSpec {
    pub id: UnitId,
    pub kind: UnitKind,
    pub timeout: Duration,
    pub policy: RestartPolicy,
    pub restart_cb: Arc<dyn Fn() -> Result<(), WatchdogError> + Send + Sync>,
}

impl RegisterSpec {
    pub fn validate(&self) -> Result<(), WatchdogError> {
        if self.id.trim().is_empty() {
            return Err(WatchdogError::InvalidConfig("id must be non-empty"));
        }
        if self.timeout < Duration::from_millis(50) {
            return Err(WatchdogError::InvalidConfig("timeout too small"));
        }
        Ok(())
    }
}

/// Internal record stored in the registry.
#[derive(Clone)]
pub struct UnitRecord {
    pub id: UnitId,
    pub kind: UnitKind,
    pub timeout: Duration,
    pub policy: RestartPolicy,
    pub restart_cb: Arc<dyn Fn() -> Result<(), WatchdogError> + Send + Sync>,
    pub last_beat: Arc<std::sync::atomic::AtomicU64>, // monotonic millis
    pub restarts: u32,
    pub next_backoff: Duration,
}

impl UnitRecord {
    pub fn new(spec: RegisterSpec) -> Self {
        let now_ms = now_instant_millis();
        let policy = spec.policy.clone(); // <-- FIX: clone so we can use twice
        Self {
            id: spec.id,
            kind: spec.kind,
            timeout: spec.timeout,
            policy,
            restart_cb: spec.restart_cb,
            last_beat: Arc::new(std::sync::atomic::AtomicU64::new(now_ms)),
            restarts: 0,
            next_backoff: spec.policy.base_backoff,
        }
    }

    #[inline]
    pub fn update_heartbeat(&self) {
        self.last_beat
            .store(now_instant_millis(), std::sync::atomic::Ordering::Relaxed);
    }

    #[inline]
    pub fn millis_since_beat(&self) -> u128 {
        let last = self.last_beat.load(std::sync::atomic::Ordering::Relaxed) as u128;
        now_instant_millis() as u128 - last
    }

    pub fn due_for_restart(&self) -> bool {
        self.millis_since_beat() >= self.timeout.as_millis()
    }

    pub fn bump_backoff(&mut self) {
        let next = self.next_backoff.saturating_mul(2);
        self.next_backoff = std::cmp::min(next, self.policy.max_backoff);
    }

    pub fn reset_backoff(&mut self) {
        self.next_backoff = self.policy.base_backoff;
    }
}

/// Thread-safe registry.
#[derive(Default)]
pub struct Registry {
    inner: RwLock<HashMap<UnitId, UnitRecord>>,
}

impl Registry {
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(HashMap::new()),
        }
    }

    pub fn insert(&self, spec: RegisterSpec) -> Result<(), WatchdogError> {
        spec.validate()?;
        let rec = UnitRecord::new(spec);
        let mut m = self.inner.write();
        if m.contains_key(&rec.id) {
            return Err(WatchdogError::DuplicateUnit(rec.id));
        }
        m.insert(rec.id.clone(), rec);
        Ok(())
    }

    pub fn heartbeat(&self, id: &str) -> Result<(), WatchdogError> {
        let m = self.inner.read();
        let rec = m.get(id).ok_or_else(|| WatchdogError::UnknownUnit(id.to_string()))?;
        rec.update_heartbeat();
        Ok(())
    }

    pub fn snapshot(&self) -> Vec<UnitRecord> {
        self.inner.read().values().cloned().collect()
    }

    pub fn with_record_mut<F: FnOnce(&mut UnitRecord)>(&self, id: &str, f: F) -> Result<(), WatchdogError> {
        let mut m = self.inner.write();
        let rec = m.get_mut(id).ok_or_else(|| WatchdogError::UnknownUnit(id.to_string()))?;
        f(rec);
        Ok(())
    }
}

#[inline]
fn now_instant_millis() -> u64 {
    static START: Lazy<std::time::Instant> = Lazy::new(std::time::Instant::now);
    let dur = START.elapsed();
    dur.as_secs() * 1_000 + u64::from(dur.subsec_millis())
}