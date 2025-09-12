//! Module 2 — Config Loader (Phase I)

use once_cell::sync::Lazy;
use regex::Regex;
use serde::Deserialize;
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("parse error: {0}")]
    Parse(String),
    #[error("validation error: {0}")]
    Invalid(String),
    #[error("env override error: {0}")]
    Env(String),
}
pub type Result<T> = std::result::Result<T, ConfigError>;

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct EngineConfig {
    pub meta: MetaCfg,
    pub logging: LoggingCfg,
    pub scheduler: SchedulerCfg,
    pub ipc: IpcCfg,
    pub plugins: PluginCfg,
    pub telemetry: TelemetryCfg,
    pub paths: PathsCfg,
    #[serde(default)]
    pub toggles: TogglesCfg,
}
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct MetaCfg { pub schema_version: u32, pub phase: String }
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct LoggingCfg { pub level: String, pub stdout: bool, pub rotate_after_mb: u64, pub max_days: u32 }
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct SchedulerCfg { pub tick_ms: u64, pub watchdog_ms: u64 }
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct IpcCfg { pub bind: String, pub auth_token: String }
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct PluginCfg {
    pub auto_load: bool,
    pub directories: Vec<String>,
    pub secure_mode_threshold: f32,
    #[serde(default)] pub max_concurrent: u32,
}
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct TelemetryCfg { pub enabled: bool, pub compress: bool }
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct PathsCfg { pub log_dir: String, #[serde(default)] pub export_dir: String }
#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
pub struct TogglesCfg { #[serde(default)] pub dns_logging: bool, #[serde(default)] pub entropy_flagger: bool, #[serde(default)] pub clipboard_events: bool }

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            meta: MetaCfg { schema_version: 1, phase: "phase1".into() },
            logging: LoggingCfg { level: "info".into(), stdout: true, rotate_after_mb: 50, max_days: 7 },
            scheduler: SchedulerCfg { tick_ms: 100, watchdog_ms: 500 },
            ipc: IpcCfg { bind: "/tmp/overflow.sock".into(), auth_token: "change_me_please".into() },
            plugins: PluginCfg { auto_load: true, directories: vec!["plugins".into()], secure_mode_threshold: 7.5, max_concurrent: 2 },
            telemetry: TelemetryCfg { enabled: true, compress: true },
            paths: PathsCfg { log_dir: "logs".into(), export_dir: "logs/snapshots".into() },
            toggles: TogglesCfg::default(),
        }
    }
}

pub fn load_from_path<P: AsRef<Path>>(path: P) -> Result<EngineConfig> {
    let bytes = fs::read(path)?;
    let text = String::from_utf8(bytes)
        .map_err(|e| ConfigError::Parse(format!("utf8 decode: {e}")))?;
    let mut cfg: EngineConfig = toml::from_str(&text)
        .map_err(|e| ConfigError::Parse(e.to_string()))?;
    apply_env_overrides(&mut cfg)?;
    validate(&cfg)?;
    Ok(cfg)
}
pub fn load_from_str(s: &str) -> Result<EngineConfig> {
    let mut cfg: EngineConfig = toml::from_str(s).map_err(|e| ConfigError::Parse(e.to_string()))?;
    apply_env_overrides(&mut cfg)?;
    validate(&cfg)?;
    Ok(cfg)
}

fn apply_env_overrides(cfg: &mut EngineConfig) -> Result<()> {
    if let Ok(level) = std::env::var("OVERFLOW_LOG_LEVEL") { if !level.trim().is_empty() { cfg.logging.level = level; } }
    if let Ok(stdout) = std::env::var("OVERFLOW_LOG_STDOUT") { cfg.logging.stdout = stdout == "1" || stdout.eq_ignore_ascii_case("true"); }
    if let Ok(bind) = std::env::var("OVERFLOW_IPC_BIND") { if !bind.trim().is_empty() { cfg.ipc.bind = bind; } }
    if let Ok(tok) = std::env::var("OVERFLOW_IPC_TOKEN") {
        if tok.len() >= 8 { cfg.ipc.auth_token = tok; } else { return Err(ConfigError::Env("OVERFLOW_IPC_TOKEN too short (<8)".into())); }
    }
    if let Ok(dir) = std::env::var("OVERFLOW_LOG_DIR") { if !dir.trim().is_empty() { cfg.paths.log_dir = dir; } }
    if let Ok(dirs) = std::env::var("OVERFLOW_PLUGIN_DIRS") {
        let v: Vec<String> = dirs.split(':').filter(|s| !s.trim().is_empty()).map(|s| s.to_string()).collect();
        if !v.is_empty() { cfg.plugins.directories = v; }
    }
    if let Ok(tick) = std::env::var("OVERFLOW_TICK_MS") {
        cfg.scheduler.tick_ms = tick.parse::<u64>().map_err(|_| ConfigError::Env("OVERFLOW_TICK_MS not an integer".into()))?;
    }
    if let Ok(wd) = std::env::var("OVERFLOW_WATCHDOG_MS") {
        cfg.scheduler.watchdog_ms = wd.parse::<u64>().map_err(|_| ConfigError::Env("OVERFLOW_WATCHDOG_MS not an integer".into()))?;
    }
    Ok(())
}

pub fn validate(cfg: &EngineConfig) -> Result<()> {
    if cfg.meta.schema_version == 0 { return Err(ConfigError::Invalid("meta.schema_version must be >=1".into())); }
    if cfg.meta.phase.trim().is_empty() { return Err(ConfigError::Invalid("meta.phase cannot be empty".into())); }

    static LEVEL_RE: Lazy<Regex> = Lazy::new(|| Regex::new("^(trace|debug|info|warn|error)$").unwrap());
    if !LEVEL_RE.is_match(&cfg.logging.level.to_lowercase()) {
        return Err(ConfigError::Invalid("logging.level must be one of trace|debug|info|warn|error".into()));
    }
    if cfg.logging.rotate_after_mb == 0 || cfg.logging.rotate_after_mb > 1024 {
        return Err(ConfigError::Invalid("logging.rotate_after_mb must be 1..=1024".into()));
    }
    if cfg.logging.max_days == 0 || cfg.logging.max_days > 365 {
        return Err(ConfigError::Invalid("logging.max_days must be 1..=365".into()));
    }

    if cfg.scheduler.tick_ms < 50 { return Err(ConfigError::Invalid("scheduler.tick_ms must be >= 50".into())); }
    if cfg.scheduler.watchdog_ms < cfg.scheduler.tick_ms {
        return Err(ConfigError::Invalid("scheduler.watchdog_ms must be >= tick_ms".into()));
    }

    if cfg.ipc.bind.trim().is_empty() { return Err(ConfigError::Invalid("ipc.bind cannot be empty".into())); }
    if cfg.ipc.auth_token.len() < 8 { return Err(ConfigError::Invalid("ipc.auth_token must be >= 8 chars".into())); }

    if cfg.plugins.max_concurrent == 0 { return Err(ConfigError::Invalid("plugins.max_concurrent must be >= 1".into())); }
    if !(0.0..=10.0).contains(&cfg.plugins.secure_mode_threshold) {
        return Err(ConfigError::Invalid("plugins.secure_mode_threshold must be in 0.0..=10.0".into()));
    }
    for d in &cfg.plugins.directories {
        if d.trim().is_empty() { return Err(ConfigError::Invalid("plugins.directories contains empty path".into())); }
    }

    ensure_dir_writable(&cfg.paths.log_dir)?;
    if !cfg.paths.export_dir.trim().is_empty() { ensure_dir_creatable(&cfg.paths.export_dir)?; }
    Ok(())
}

fn ensure_dir_writable(path: &str) -> Result<()> {
    let pb = PathBuf::from(path);
    if !pb.exists() { fs::create_dir_all(&pb)?; }
    let probe = pb.join(".wprobe");
    match fs::File::create(&probe) {
        Ok(_) => { let _ = fs::remove_file(&probe); Ok(()) }
        Err(e) => Err(ConfigError::Invalid(format!("cannot write to {}: {e}", pb.display()))),
    }
}
fn ensure_dir_creatable(path: &str) -> Result<()> {
    let pb = PathBuf::from(path);
    if pb.exists() { return Ok(()); }
    fs::create_dir_all(&pb).map_err(|e| ConfigError::Invalid(format!("cannot create {}: {e}", pb.display())))
}

pub fn as_kv(cfg: &EngineConfig) -> HashMap<&'static str, String> {
    let mut m = HashMap::new();
    m.insert("meta.schema_version", cfg.meta.schema_version.to_string());
    m.insert("meta.phase", cfg.meta.phase.clone());
    m.insert("logging.level", cfg.logging.level.clone());
    m.insert("logging.stdout", cfg.logging.stdout.to_string());
    m.insert("scheduler.tick_ms", cfg.scheduler.tick_ms.to_string());
    m.insert("scheduler.watchdog_ms", cfg.scheduler.watchdog_ms.to_string());
    m.insert("ipc.bind", cfg.ipc.bind.clone());
    m.insert("plugins.auto_load", cfg.plugins.auto_load.to_string());
    m.insert("plugins.secure_mode_threshold", cfg.plugins.secure_mode_threshold.to_string());
    m.insert("paths.log_dir", cfg.paths.log_dir.clone());
    m
}