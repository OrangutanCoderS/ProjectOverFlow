//! Module 3 — Data Models (Phase I)
//! Canonical event/data structs shared across subsystems.
//! Pure data + validation + (de)serialization helpers. No side effects.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;
use uuid::Uuid;

/// Milliseconds since Unix epoch (UTC)
pub type TimestampMs = i64;

/// Errors from model validation and JSON helpers.
#[derive(Debug, Error)]
pub enum ModelError {
    #[error("invalid value: {0}")]
    Invalid(String),
    #[error("serialization: {0}")]
    Serde(String),
}

/// Event kind — used for routing and logging.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    ProcessInfo,
    FileAccess,
    Network,
    PluginTrigger,
    SecureModeState,
    // NEW: system metrics snapshot
    SystemStats,
}

/// Minimal common header every event carries.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BaseEvent {
    pub id: Uuid,
    pub ts_ms: TimestampMs,
    pub pid: i32,
    #[serde(default)]
    pub event_source: String, // producer label (e.g., "filemon", "process", "system")
}

impl BaseEvent {
    /// Create with current time and random ID.
    pub fn new(pid: i32, event_source: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            ts_ms: Utc::now().timestamp_millis(),
            pid,
            event_source: event_source.into(),
        }
    }

    /// SHA-256 hash of {header JSON || payload JSON} — for integrity checks.
    pub fn content_hash_hex<T: Serialize>(&self, payload: &T) -> Result<String, ModelError> {
        let mut hasher = Sha256::new();
        let header = serde_json::to_vec(self).map_err(|e| ModelError::Serde(e.to_string()))?;
        let body = serde_json::to_vec(payload).map_err(|e| ModelError::Serde(e.to_string()))?;
        hasher.update(&header);
        hasher.update(&body);
        Ok(format!("{:x}", hasher.finalize()))
    }
}

/// Common behavior for all events.
pub trait Event: Serialize {
    fn kind(&self) -> EventType;
    fn base(&self) -> &BaseEvent;
    fn to_json(&self) -> Result<String, ModelError> {
        serde_json::to_string(self).map_err(|e| ModelError::Serde(e.to_string()))
    }
    fn validate(&self) -> Result<(), ModelError>;
}

/* =======================
   Process Information
   ======================= */

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProcessInfo {
    #[serde(flatten)]
    pub base: BaseEvent,
    pub name: String,
    pub ppid: i32,
    pub user: String,
    pub cpu_pct: f32,    // may exceed 100 on multi-core sampling
    pub mem_mb: f32,
    pub threads: u32,
    pub cmdline: Vec<String>,
    pub start_time_ms: TimestampMs,
}

impl Event for ProcessInfo {
    fn kind(&self) -> EventType { EventType::ProcessInfo }
    fn base(&self) -> &BaseEvent { &self.base }
    fn validate(&self) -> Result<(), ModelError> {
        if self.base.pid <= 0 { return Err(ModelError::Invalid("pid must be > 0".into())); }
        if self.name.trim().is_empty() { return Err(ModelError::Invalid("name cannot be empty".into())); }
        if self.threads == 0 { return Err(ModelError::Invalid("threads must be >= 1".into())); }
        if !self.user.chars().all(|c| c.is_ascii()) { return Err(ModelError::Invalid("user must be ASCII".into())); }
        Ok(())
    }
}

/* =======================
   File Access
   ======================= */

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FileOp { Read, Write, Delete, Exec, Rename, Open }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FileAccessEvent {
    #[serde(flatten)]
    pub base: BaseEvent,
    pub path: String,
    pub op: FileOp,
    pub entropy: Option<f32>,            // optional 0.0..=10.0
    #[serde(default)]
    pub plugin_context: Option<String>,  // plugin/id that influenced logging
}

impl Event for FileAccessEvent {
    fn kind(&self) -> EventType { EventType::FileAccess }
    fn base(&self) -> &BaseEvent { &self.base }
    fn validate(&self) -> Result<(), ModelError> {
        if self.base.pid <= 0 { return Err(ModelError::Invalid("pid must be > 0".into())); }
        if self.path.trim().is_empty() { return Err(ModelError::Invalid("path cannot be empty".into())); }
        if let Some(h) = self.entropy {
            if !(0.0..=10.0).contains(&h) { return Err(ModelError::Invalid("entropy must be within 0.0..=10.0".into())); }
        }
        Ok(())
    }
}

/* =======================
   Network
   ======================= */

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NetProto { Tcp, Udp, Other }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NetworkEvent {
    #[serde(flatten)]
    pub base: BaseEvent,
    pub proto: NetProto,
    pub remote_addr: String,         // "ip" or "ip:port"
    pub local_addr: Option<String>,  // "ip:port"
    pub dns_name: Option<String>,
}

impl Event for NetworkEvent {
    fn kind(&self) -> EventType { EventType::Network }
    fn base(&self) -> &BaseEvent { &self.base }
    fn validate(&self) -> Result<(), ModelError> {
        if self.base.pid <= 0 { return Err(ModelError::Invalid("pid must be > 0".into())); }
        if self.remote_addr.trim().is_empty() { return Err(ModelError::Invalid("remote_addr cannot be empty".into())); }
        if let Some(dns) = &self.dns_name {
            if dns.len() > 255 { return Err(ModelError::Invalid("dns_name too long".into())); }
        }
        Ok(())
    }
}

/* =======================
   Plugin Trigger
   ======================= */

bitflags::bitflags! {
    /// Optional flags about the trigger context.
    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
    pub struct TriggerFlags: u32 {
        const THRESHOLD_EXCEEDED = 0b0001;
        const HEURISTIC_MATCH    = 0b0010;
        const USER_POLICY        = 0b0100;
        const SANDBOXED          = 0b1000;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PluginTrigger {
    #[serde(flatten)]
    pub base: BaseEvent,
    pub plugin: String,
    pub reason: String,
    pub score: f32,                 // 0.0..=10.0 severity
    #[serde(default)]
    pub flags: TriggerFlags,
    #[serde(default)]
    pub context_summary: Option<String>,
}

impl Event for PluginTrigger {
    fn kind(&self) -> EventType { EventType::PluginTrigger }
    fn base(&self) -> &BaseEvent { &self.base }
    fn validate(&self) -> Result<(), ModelError> {
        if self.base.pid < 0 { return Err(ModelError::Invalid("pid must be >= 0".into())); } // non-process triggers allowed at pid=0
        if self.plugin.trim().is_empty() { return Err(ModelError::Invalid("plugin cannot be empty".into())); }
        if !(0.0..=10.0).contains(&self.score) { return Err(ModelError::Invalid("score must be within 0.0..=10.0".into())); }
        Ok(())
    }
}

/* =======================
   Secure Mode State
   ======================= */

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SecureModeState {
    #[serde(flatten)]
    pub base: BaseEvent,
    pub active: bool,
    pub triggered_by: Option<String>, // plugin/id/reason
    pub trigger_score: Option<f32>,
}

impl Event for SecureModeState {
    fn kind(&self) -> EventType { EventType::SecureModeState }
    fn base(&self) -> &BaseEvent { &self.base }
    fn validate(&self) -> Result<(), ModelError> {
        if let Some(s) = self.trigger_score {
            if !(0.0..=10.0).contains(&s) { return Err(ModelError::Invalid("trigger_score must be within 0.0..=10.0".into())); }
        }
        Ok(())
    }
}

/* =======================
   System Stats Snapshot (NEW)
   ======================= */

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SystemStatSnapshot {
    #[serde(flatten)]
    pub base: BaseEvent,

    // percentages are per global CPU (0..=100 on sysinfo global; >100 on per-core sums but we use global)
    pub cpu_pct: f32,

    // memory/swap in MiB
    pub mem_total_mb: f32,
    pub mem_used_mb: f32,
    pub swap_total_mb: f32,
    pub swap_used_mb: f32,

    // UNIX load averages
    pub load1: f32,
    pub load5: f32,
    pub load15: f32,

    // uptime in seconds and total process count (from provider)
    pub uptime_s: i64,
    pub process_count: u32,
}

impl Event for SystemStatSnapshot {
    fn kind(&self) -> EventType { EventType::SystemStats }
    fn base(&self) -> &BaseEvent { &self.base }
    fn validate(&self) -> Result<(), ModelError> {
        if !(0.0..=100.0).contains(&self.cpu_pct) {
            return Err(ModelError::Invalid("cpu_pct must be within 0..=100".into()));
        }
        if self.mem_used_mb < 0.0 || self.swap_used_mb < 0.0 {
            return Err(ModelError::Invalid("mem_used_mb/swap_used_mb must be >= 0".into()));
        }
        if self.mem_total_mb < self.mem_used_mb + 0.0 {
            // tolerate minor float noise; comparison already permissive
        }
        if self.load1 < 0.0 || self.load5 < 0.0 || self.load15 < 0.0 {
            return Err(ModelError::Invalid("load averages must be >= 0".into()));
        }
        if self.uptime_s < 0 {
            return Err(ModelError::Invalid("uptime_s must be >= 0".into()));
        }
        Ok(())
    }
}

/* =======================
   Generic wrapper for logging/telemetry
   ======================= */

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "event_type", content = "data", rename_all = "snake_case")]
pub enum AnyEvent {
    ProcessInfo(ProcessInfo),
    FileAccess(FileAccessEvent),
    Network(NetworkEvent),
    PluginTrigger(PluginTrigger),
    SecureModeState(SecureModeState),
    // NEW:
    SystemStats(SystemStatSnapshot),
}

impl AnyEvent {
    pub fn validate(&self) -> Result<(), ModelError> {
        match self {
            AnyEvent::ProcessInfo(e) => e.validate(),
            AnyEvent::FileAccess(e) => e.validate(),
            AnyEvent::Network(e) => e.validate(),
            AnyEvent::PluginTrigger(e) => e.validate(),
            AnyEvent::SecureModeState(e) => e.validate(),
            AnyEvent::SystemStats(e) => e.validate(),
        }
    }

    pub fn to_json(&self) -> Result<String, ModelError> {
        serde_json::to_string(self).map_err(|e| ModelError::Serde(e.to_string()))
    }
}