//! Module 22 — Plugin API
//! Centralized, secure façade for plugins to emit alerts and log actions.
//! - Singleton `PluginApi` with quotas
//! - Thin wrapper fns for plugin convenience
//! - No compile-time dependency on your event model: accept JSON payloads

use anyhow::Result;
use chrono::{DateTime, Utc};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::HashMap,
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
    sync::Mutex,
    time::{Duration, Instant},
};
use thiserror::Error;

/// Log file path (append-only, one JSON line per event)
const LOG_PATH: &str = "logs/plugin_api_log.json";

/// Global singleton
static API: Lazy<PluginApi> = Lazy::new(PluginApi::new);

/// Public facade (singleton)
pub struct PluginApi {
    cfg: ApiConfig,
    inner: Mutex<Inner>,
}

/// Configuration (copyable)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    /// Sliding window length for quotas.
    pub window: Duration,
    /// Max alerts per plugin per window.
    pub max_alerts_per_window: u32,
    /// Max logs per plugin per window.
    pub max_logs_per_window: u32,
    /// Optional override for log file path (mainly for tests).
    pub log_path: Option<PathBuf>,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            window: Duration::from_secs(60),
            max_alerts_per_window: 60,
            max_logs_per_window: 1000,
            log_path: None,
        }
    }
}

/// Per-plugin counters for quotas
#[derive(Debug, Clone)]
struct Counters {
    window_start: Instant,
    alerts: u32,
    logs: u32,
}

impl Counters {
    fn new(now: Instant) -> Self {
        Self {
            window_start: now,
            alerts: 0,
            logs: 0,
        }
    }

    fn maybe_reset(&mut self, now: Instant, window: Duration) {
        if now.duration_since(self.window_start) >= window {
            self.window_start = now;
            self.alerts = 0;
            self.logs = 0;
        }
    }
}

#[derive(Default)]
struct Inner {
    cfg: ApiConfig,                      // live config snapshot
    counters: HashMap<String, Counters>, // by plugin name
}

/// Public error surface
#[derive(Debug, Error)]
pub enum PluginApiError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialize error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("quota exceeded for plugin '{plugin}' on {kind}")]
    QuotaExceeded { plugin: String, kind: &'static str },
    #[error("invalid input: {0}")]
    Invalid(&'static str),
}

#[derive(Debug, Serialize)]
struct LogRow<'a> {
    timestamp: DateTime<Utc>,
    kind: &'static str, // "alert" | "log"
    plugin: &'a str,
    reason: Option<&'a str>,
    action: Option<&'a str>,
    payload: Option<&'a Value>,
}

impl PluginApi {
    pub fn new() -> Self {
        Self {
            cfg: ApiConfig::default(),
            inner: Mutex::new(Inner {
                cfg: ApiConfig::default(),
                counters: HashMap::new(),
            }),
        }
    }

    /// Access the global singleton
    pub fn global() -> &'static Self {
        &API
    }

    /// Replace runtime config (e.g., from daemon). Safe & atomic.
    pub fn set_config(&self, cfg: ApiConfig) {
        let mut g = self.inner.lock().expect("poisoned");
        g.cfg = cfg.clone();
        g.counters.clear(); // reset counters when config changes

        // Keep top-level copy in sync for read_config()
        let p = self as *const Self as *mut Self;
        unsafe { (*p).cfg = cfg };
    }

    /// Read config snapshot (no lock).
    pub fn read_config(&self) -> ApiConfig {
        self.cfg.clone()
    }

    /// Core: emit an alert with structured payload.
    pub fn emit_alert_json(&self, plugin: &str, reason: &str, payload: &Value) -> Result<()> {
        self.guard_nonempty(plugin, "plugin")?;
        self.guard_nonempty(reason, "reason")?;

        let log_path = {
            let mut g = self.inner.lock().expect("poisoned");
            let now = Instant::now();

            let window = g.cfg.window;
            let max_alerts = g.cfg.max_alerts_per_window;
            let log_path = g.cfg.log_path.clone();

            let ent = g
                .counters
                .entry(plugin.to_string())
                .or_insert_with(|| Counters::new(now));
            ent.maybe_reset(now, window);

            if ent.alerts >= max_alerts {
                return Err(PluginApiError::QuotaExceeded {
                    plugin: plugin.to_string(),
                    kind: "alerts",
                }
                .into());
            }
            ent.alerts += 1;
            log_path
        };

        self.append_row(
            log_path,
            &LogRow {
                timestamp: Utc::now(),
                kind: "alert",
                plugin,
                reason: Some(reason),
                action: None,
                payload: Some(payload),
            },
        )?;
        Ok(())
    }

    /// Core: append a log entry (lightweight).
    pub fn log_event(&self, plugin: &str, action: &str) -> Result<()> {
        self.guard_nonempty(plugin, "plugin")?;
        self.guard_nonempty(action, "action")?;

        let log_path = {
            let mut g = self.inner.lock().expect("poisoned");
            let now = Instant::now();

            let window = g.cfg.window;
            let max_logs = g.cfg.max_logs_per_window;
            let log_path = g.cfg.log_path.clone();

            let ent = g
                .counters
                .entry(plugin.to_string())
                .or_insert_with(|| Counters::new(now));
            ent.maybe_reset(now, window);

            if ent.logs >= max_logs {
                return Err(PluginApiError::QuotaExceeded {
                    plugin: plugin.to_string(),
                    kind: "logs",
                }
                .into());
            }
            ent.logs += 1;
            log_path
        };

        self.append_row(
            log_path,
            &LogRow {
                timestamp: Utc::now(),
                kind: "log",
                plugin,
                reason: None,
                action: Some(action),
                payload: None,
            },
        )?;
        Ok(())
    }

    /// Helper: simple text alert (no JSON payload).
    pub fn emit_alert_text(&self, plugin: &str, reason: &str, message: &str) -> Result<()> {
        let payload = serde_json::json!({ "message": message });
        self.emit_alert_json(plugin, reason, &payload)
    }

    fn guard_nonempty(&self, s: &str, field: &'static str) -> Result<()> {
        if s.trim().is_empty() {
            return Err(PluginApiError::Invalid(field).into());
        }
        Ok(())
    }

    fn append_row(&self, log_path: Option<PathBuf>, row: &LogRow<'_>) -> Result<()> {
        let path = log_path.unwrap_or_else(|| PathBuf::from(LOG_PATH));
        let line = serde_json::to_string(row)?;
        let mut fh = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)?;
        writeln!(fh, "{line}")?;
        Ok(())
    }
}

/* ---------------------------
   Thin wrappers for plugins
   --------------------------- */

pub fn emit_alert_json(plugin: &str, reason: &str, payload: &Value) -> Result<()> {
    PluginApi::global().emit_alert_json(plugin, reason, payload)
}

pub fn emit_alert_text(plugin: &str, reason: &str, message: &str) -> Result<()> {
    PluginApi::global().emit_alert_text(plugin, reason, message)
}

pub fn log_event(plugin: &str, action: &str) -> Result<()> {
    PluginApi::global().log_event(plugin, action)
}

pub fn read_config() -> ApiConfig {
    PluginApi::global().read_config()
}

pub fn set_config(cfg: ApiConfig) {
    PluginApi::global().set_config(cfg)
}

/* ---------------------------
   Tests
   --------------------------- */

#[cfg(test)]
mod self_tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn config_roundtrip_and_logging() {
        let tmp = tempfile_path();
        let _ = fs::remove_file(&tmp); // cleanup before

        let cfg = ApiConfig {
            log_path: Some(tmp.clone()),
            ..Default::default()
        };
        set_config(cfg.clone());
        assert_eq!(read_config().max_alerts_per_window, cfg.max_alerts_per_window);

        log_event("p1", "started").unwrap();
        emit_alert_text("p1", "anomaly", "something happened").unwrap();

        let s = fs::read_to_string(&tmp).unwrap();
        let lines = s.lines().count();
        assert_eq!(lines, 2);

        let _ = fs::remove_file(&tmp); // cleanup after
    }

    #[test]
    fn quotas_enforced() {
        let tmp = tempfile_path();
        let _ = fs::remove_file(&tmp); // cleanup before

        set_config(ApiConfig {
            window: Duration::from_secs(3600),
            max_alerts_per_window: 1,
            max_logs_per_window: 1,
            log_path: Some(tmp.clone()),
        });

        emit_alert_text("p2", "r", "m").unwrap();
        let e = emit_alert_text("p2", "r", "m").unwrap_err();
        assert!(format!("{e}").contains("quota exceeded"));

        log_event("p3", "a").unwrap();
        let e = log_event("p3", "b").unwrap_err();
        assert!(format!("{e}").contains("quota exceeded"));

        let _ = fs::remove_file(&tmp); // cleanup after
    }

    fn tempfile_path() -> PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!(
            "plugin_api_test_{}.json",
            Utc::now().timestamp_nanos_opt().unwrap_or(0)
        ));
        p
    }
}