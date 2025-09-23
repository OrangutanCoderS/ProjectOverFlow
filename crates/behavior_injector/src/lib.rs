//! behavior_injector — Phase II intervention dispatcher (Rust)
//!
//! Goals:
//! - Accept normalized intervention requests.
//! - Validate targets / policies without touching legacy crates.
//! - Dispatch to registered action handlers (kill / suspend / throttle / fake_output / …).
//! - Log results atomically (JSONL) to `logs/intervention_log.json` (default).
//! - Sandbox risky actions when requested (feature-gated).
//!
//! Design keeps existing OverFlow crates untouched.
//! Integrations are optional via features to avoid breaking old code.
//!
//! Security posture: no unsafe by default; controlled use behind small wrappers where needed.

#![deny(unsafe_code)]
#![deny(clippy::all, clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

mod model;
mod registry;
mod resultlog;
mod errors;
mod util;

pub use model::{Action, InterventionRequest, InterventionResult};
pub use errors::{InjectorError, ValidationError, ExecutionError};
pub use registry::{ActionHandler, ActionRegistry, DEFAULT_REGISTRY};
pub use resultlog::{InterventionLogger, LoggerTarget, JsonlLogger};
pub use util::{utc_timestamp, ProtectedPidPolicy};

use parking_lot::RwLock;
use once_cell::sync::Lazy;
use tracing::{warn, info};

/// Global configuration (simple, local; caller may pass richer structs in future).
#[derive(Clone, Debug)]
pub struct InjectorConfig {
    /// Dry-run mode (simulate, do not execute).
    pub dry_run: bool,
    /// Mirror into plugin_log.json too (future toggle).
    pub unify_logs: bool,
    /// Optional protected PID policy (self-protect the daemon & critical PIDs).
    pub protected_policy: ProtectedPidPolicy,
}

/// Default config: live-mode = false, separate intervention log, basic protections on.
impl Default for InjectorConfig {
    fn default() -> Self {
        Self {
            dry_run: false,
            unify_logs: false, // per decision: interventions go to intervention_log.json by default
            protected_policy: ProtectedPidPolicy::default(),
        }
    }
}

/// Core injector object — owns registry + logger + config.
pub struct BehaviorInjector<L: InterventionLogger> {
    registry: ActionRegistry,
    logger: L,
    config: InjectorConfig,
}

/// Public factory for the injector with default registry + JSONL logger.
impl BehaviorInjector<JsonlLogger> {
    /// Creates a default injector with:
    /// - default action registry (kill/suspend/throttle/fake_output placeholders)
    /// - JSONL logger to logs/intervention_log.json (and optionally plugin_log.json)
    pub fn new_default(config: InjectorConfig) -> anyhow::Result<Self> {
        let logger = JsonlLogger::new_default()?;
        Ok(Self {
            registry: DEFAULT_REGISTRY.read().clone(),
            logger,
            config,
        })
    }
}

impl<L: InterventionLogger> BehaviorInjector<L> {
    /// Main entry point: validate → resolve → execute → log → return result.
    /// Never panics; returns InterventionResult with success/failure/dry-run.
    pub fn dispatch_intervention(&self, req: InterventionRequest) -> InterventionResult {
        // 1) Validation
        if let Err(e) = req.validate(&self.config.protected_policy) {
            let res = InterventionResult::failure(&req, "validation", &e.to_string());
            self.safe_log(&res);
            return res;
        }

        // 2) Resolution
        let Some(handler) = self.registry.get(&req.action) else {
            let res = InterventionResult::failure(&req, "resolve", "unknown_action");
            self.safe_log(&res);
            return res;
        };

        // 3) Execution or Dry-run
        let outcome = if self.config.dry_run {
            info!("DRY-RUN: would execute action {:?}", req.action);
            Ok("dry-run".to_string())
        } else {
            handler.execute(&req).map(|_| "success".to_string())
        };

        // 4) Result + Logging
        let res = match outcome {
            Ok(status) if status == "dry-run" => {
                InterventionResult::dry_run(&req, handler.module_name())
            }
            Ok(_) => InterventionResult::success(&req, handler.module_name()),
            Err(err) => {
                InterventionResult::exec_failure(&req, handler.module_name(), &err.to_string())
            }
        };

        self.safe_log(&res);
        res
    }

    fn safe_log(&self, res: &InterventionResult) {
        if let Err(err) = self.logger.append(res, self.config.unify_logs) {
            warn!(error = %err, "behavior_injector: log write failed");
        }
    }
}

/// Thread-safe singleton accessor (optional utility)
pub static INJECTOR_SINGLETON: Lazy<RwLock<Option<BehaviorInjector<JsonlLogger>>>> =
    Lazy::new(|| RwLock::new(None));

/// Install a global injector (useful for daemons/tests). Idempotent replacement.
pub fn install_global_injector(inj: BehaviorInjector<JsonlLogger>) {
    let mut guard = INJECTOR_SINGLETON.write();
    *guard = Some(inj);
}

/// Dispatch via global injector (for callers who prefer no handles).
pub fn dispatch_via_global(req: InterventionRequest) -> Result<InterventionResult, InjectorError> {
    let guard = INJECTOR_SINGLETON.read();
    let Some(ref inj) = *guard else {
        return Err(InjectorError::NotInitialized);
    };
    Ok(inj.dispatch_intervention(req))
}
