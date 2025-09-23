use crate::{
    error::WatchdogError,
    escalation::{EscalationEvent, EscalationHook, NoopHook},
    registry::{Registry, RegisterSpec, UnitId},
};
use chrono::{DateTime, Utc};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::{Duration, Instant},
};

/// Public configuration for the Watchdog.
#[derive(Debug, Clone)]
pub struct WatchdogConfig {
    /// How often the monitor loop scans all units.
    pub poll_interval: Duration,
    /// Max number of units to process per iteration (cooperative).
    pub batch: usize,
}

impl Default for WatchdogConfig {
    fn default() -> Self {
        Self {
            poll_interval: Duration::from_millis(50),
            batch: 256,
        }
    }
}

/// Watchdog entry point.
pub struct Watchdog {
    cfg: WatchdogConfig,
    registry: Arc<Registry>,
    hook: Arc<dyn EscalationHook>,
    running: Arc<AtomicBool>,
    handle: Option<thread::JoinHandle<()>>,
}

impl Watchdog {
    pub fn new(cfg: WatchdogConfig) -> Self {
        Self {
            cfg,
            registry: Arc::new(Registry::new()),
            hook: Arc::new(NoopHook),
            running: Arc::new(AtomicBool::new(false)),
            handle: None,
        }
    }

    pub fn with_escalation_hook(mut self, hook: Arc<dyn EscalationHook>) -> Self {
        self.hook = hook;
        self
    }

    /// Start supervisor thread. Idempotent.
    pub fn start(&mut self) -> Result<(), WatchdogError> {
        if self.running.swap(true, Ordering::SeqCst) {
            return Ok(());
        }
        let cfg = self.cfg.clone();
        let reg = self.registry.clone();
        let running = self.running.clone();
        let hook = self.hook.clone();

        self.handle = Some(thread::spawn(move || {
            let mut next_idx: usize = 0;
            while running.load(Ordering::Relaxed) {
                let ts = Utc::now();
                let snapshot = reg.snapshot();
                if snapshot.is_empty() {
                    thread::sleep(cfg.poll_interval);
                    continue;
                }
                // Cooperative: process at most `batch` entries per tick.
                let end = std::cmp::min(next_idx + cfg.batch, snapshot.len());
                for rec in &snapshot[next_idx..end] {
                    // Check timeout
                    if rec.due_for_restart() {
                        let since = rec.millis_since_beat();
                        hook.on_event(ts, EscalationEvent::HeartbeatMissed {
                            id: &rec.id, since_ms: since
                        });
                        let id = rec.id.clone();
                        // Evaluate restarts under write lock (mut update)
                        let _ = reg.with_record_mut(&id, |r| {
                            // If already exhausted
                            if r.restarts >= r.policy.max_restarts {
                                hook.on_event(ts, EscalationEvent::MaxRestartExceeded {
                                    id: &r.id, restarts: r.restarts
                                });
                                return;
                            }
                            // Respect backoff by delaying the next allowed attempt.
                            // Here we simply sleep small backoff in this thread (bounded batch).
                            // In a larger system, schedule-at would be preferred.
                            let backoff = r.next_backoff;
                            r.restarts += 1;
                            r.bump_backoff();
                            drop(running.clone()); // keep clippy happy

                            // Execute restart callback out of the lock.
                        });
                        // Call restart outside the lock; fetch the latest (best-effort).
                        // We allow a small sleep for backoff to avoid hot-loop restarts.
                        // This makes the loop predictable and avoids stampedes.
                        thread::sleep(rec.next_backoff);
                        match (rec.restart_cb)() {
                            Ok(()) => {
                                // Record restart + recovery
                                let _ = reg.with_record_mut(&id, |r| {
                                    r.update_heartbeat();
                                });
                                hook.on_event(ts, EscalationEvent::Restarted {
                                    id: &rec.id, restarts: rec.restarts + 1
                                });
                            }
                            Err(e) => {
                                // Report failure but continue loop.
                                hook.on_event(ts, EscalationEvent::MaxRestartExceeded {
                                    id: &rec.id, restarts: rec.restarts + 1
                                });
                                eprintln!("watchdog restart failure for {}: {e}", rec.id);
                            }
                        }
                    } else {
                        // Consider notifying recovery if previously timing out: keep simple.
                    }
                }

                // Advance window and sleep.
                next_idx = if end >= snapshot.len() { 0 } else { end };
                thread::sleep(cfg.poll_interval);
            }
        }));
        Ok(())
    }

    /// Stop supervisor; waits for thread to join.
    pub fn stop(&mut self) {
        if !self.running.swap(false, Ordering::SeqCst) {
            return;
        }
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }

    /// Register a new unit with restart callback.
    pub fn register(&self, spec: RegisterSpec) -> Result<(), WatchdogError> {
        self.registry.insert(spec)
    }

    /// Heartbeat from a running unit.
    pub fn heartbeat(&self, id: &str) -> Result<(), WatchdogError> {
        self.registry.heartbeat(id)
    }

    /// Expose registry snapshot for observability/testing.
    pub fn snapshot(&self) -> Vec<crate::registry::UnitRecord> {
        self.registry.snapshot()
    }
}

impl Drop for Watchdog {
    fn drop(&mut self) {
        self.stop();
    }
}
