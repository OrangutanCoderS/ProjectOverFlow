//! Module 21 - Plugin Engine
//! Provides secure dynamic loading, lifecycle management, and invocation of plugins.
//! Rust + C ABI support. Hardened with version checks and logging.

use anyhow::Result;
use chrono::Utc;
use libloading::{Library, Symbol};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    ffi::{CStr, CString},
    fs::OpenOptions,
    io::Write,
    path::PathBuf,
    sync::Mutex,
};
use thiserror::Error;
use libc; // ✅ required for C types

/// Log file for plugin events
static LOG_PATH: &str = "logs/plugin_log.json";

/// Global lock for registry
static PLUGIN_REGISTRY: Lazy<Mutex<PluginRegistry>> =
    Lazy::new(|| Mutex::new(PluginRegistry::new()));

/// Error types
#[derive(Debug, Error)]
pub enum PluginError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Dynamic load error: {0}")]
    Load(String),

    #[error("Symbol lookup error: {0}")]
    Symbol(String),

    #[error("ABI mismatch: expected {expected}, found {found}")]
    AbiMismatch { expected: u32, found: u32 },

    #[error("Execution error: {0}")]
    Exec(String),
}

/// Plugin metadata contract
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMeta {
    pub name: String,
    pub version: String,
    pub abi: u32, // ABI version expected by engine
}

/// A loaded plugin instance
struct PluginInstance {
    lib: Library,
    meta: PluginMeta,
}

/// Registry of plugins
struct PluginRegistry {
    plugins: HashMap<String, PluginInstance>,
}

impl PluginRegistry {
    fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }
}

/// C ABI function signature for plugin metadata
type GetMetaFn = unsafe extern "C" fn() -> *const libc::c_char;

/// C ABI function signature for plugin run
type RunFn = unsafe extern "C" fn(*const libc::c_char) -> i32;

/// Public API: load a plugin
pub fn load_plugin(path: PathBuf) -> Result<PluginMeta> {
    // SAFETY: Loading a dynamic library is unsafe because its validity is external.
    let lib = unsafe { Library::new(&path) }
        .map_err(|e| PluginError::Load(e.to_string()))?;

    unsafe {
        // SAFETY: Symbol must exist and match expected signature.
        let get_meta: Symbol<GetMetaFn> = lib
            .get(b"plugin_get_meta\0")
            .map_err(|e| PluginError::Symbol(e.to_string()))?;

        let meta_str = CStr::from_ptr(get_meta());
        let meta_json = meta_str.to_str().map_err(|_| {
            PluginError::Exec("Invalid UTF-8 in plugin metadata".into())
        })?;
        let meta: PluginMeta = serde_json::from_str(meta_json)?;

        // ABI check
        if meta.abi != 1 {
            return Err(PluginError::AbiMismatch {
                expected: 1,
                found: meta.abi,
            }
            .into());
        }

        // Insert into registry
        let mut reg = PLUGIN_REGISTRY.lock().unwrap();
        reg.plugins.insert(
            meta.name.clone(),
            PluginInstance {
                lib,
                meta: meta.clone(),
            },
        );

        log_event(&meta, "loaded")?;
        Ok(meta)
    }
}

/// Public API: run a plugin by name
pub fn run_plugin(name: &str, input: &str) -> Result<i32> {
    let reg = PLUGIN_REGISTRY.lock().unwrap();
    let plugin = reg.plugins.get(name).ok_or_else(|| {
        PluginError::Exec(format!("Plugin {name} not found"))
    })?;

    unsafe {
        // SAFETY: Plugin must export `plugin_run` with correct signature.
        let run_fn: Symbol<RunFn> = plugin
            .lib
            .get(b"plugin_run\0")
            .map_err(|e| PluginError::Symbol(e.to_string()))?;

        let c_input = CString::new(input).unwrap();
        let code = run_fn(c_input.as_ptr());

        log_event(&plugin.meta, "executed")?;
        Ok(code)
    }
}

/// Append plugin event to log
fn log_event(meta: &PluginMeta, action: &str) -> Result<()> {
    let serialized = serde_json::to_string(&serde_json::json!({
        "timestamp": Utc::now(),
        "plugin": meta.name,
        "version": meta.version,
        "abi": meta.abi,
        "action": action,
    }))?;

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(LOG_PATH)?;
    writeln!(file, "{}", serialized)?;
    Ok(())
}