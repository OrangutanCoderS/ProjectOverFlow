//! Module 27 — Watchdog
//! Lightweight supervisor for threads/process-like units with heartbeats,
//! restart policy, exponential backoff, and optional escalation hooks.
//!
//! Defaults are safe and isolated; telemetry/logging sinks are optional features.

mod error;
mod registry;
mod supervisor;
mod escalation;

pub use error::WatchdogError;
pub use registry::{UnitId, UnitKind, RestartPolicy, RegisterSpec};
pub use supervisor::{Watchdog, WatchdogConfig};

/// Convenience re-export of the escalation trait for custom hooks.
pub use escalation::{EscalationHook, EscalationEvent};
