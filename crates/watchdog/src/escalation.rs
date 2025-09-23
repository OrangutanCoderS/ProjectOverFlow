use chrono::{DateTime, Utc};

/// Signals the severity/context for an escalation or notable event.
#[derive(Debug, Clone)]
pub enum EscalationEvent<'a> {
    Restarted { id: &'a str, restarts: u32 },
    MaxRestartExceeded { id: &'a str, restarts: u32 },
    HeartbeatMissed { id: &'a str, since_ms: u128 },
    Recovered { id: &'a str },
}

/// Pluggable escalation hook. Default is `NoopHook`.
pub trait EscalationHook: Send + Sync + 'static {
    fn on_event(&self, _ts: DateTime<Utc>, _ev: EscalationEvent<'_>) {}
}

/// Default: do nothing.
#[derive(Debug, Default)]
pub struct NoopHook;

impl EscalationHook for NoopHook {}

#[cfg(feature = "logging")]
mod log_hook {
    use super::*;
    use logs::{LogFormat, LogWriter, config::LogConfig};
    use serde_json::json;
    use std::sync::Arc;

    /// Simple logger-backed escalation hook.
    #[derive(Clone)]
    pub struct LoggingHook {
        writer: Arc<LogWriter>,
    }

    impl LoggingHook {
        pub fn new(writer: LogWriter) -> Self {
            Self { writer: Arc::new(writer) }
        }
    }

    impl EscalationHook for LoggingHook {
        fn on_event(&self, ts: DateTime<Utc>, ev: EscalationEvent<'_>) {
            let (lvl, msg, payload) = match ev {
                EscalationEvent::Restarted { id, restarts } => (
                    "WARN",
                    "watchdog: unit restarted",
                    json!({ "id": id, "restarts": restarts })
                ),
                EscalationEvent::MaxRestartExceeded { id, restarts } => (
                    "ERROR",
                    "watchdog: max restarts exceeded",
                    json!({ "id": id, "restarts": restarts })
                ),
                EscalationEvent::HeartbeatMissed { id, since_ms } => (
                    "WARN",
                    "watchdog: heartbeat missed",
                    json!({ "id": id, "since_ms": since_ms })
                ),
                EscalationEvent::Recovered { id } => (
                    "INFO",
                    "watchdog: unit recovered",
                    json!({ "id": id })
                ),
            };
            let _ = self.writer.log_json(lvl, "watchdog", msg, Some(&payload));
        }
    }

    pub use LoggingHook as DefaultLoggingHook;
}

#[cfg(feature = "logging")]
pub use log_hook::DefaultLoggingHook;
