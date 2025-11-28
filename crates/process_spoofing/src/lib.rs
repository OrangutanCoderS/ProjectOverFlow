//! Process Spoofing Layer (Medium Scope)
//! Safe, reversible, auditable runtime identity spoofing.
//! Uses IPC between interposer shim (C) and SpoofManager (Rust).

pub mod manager;
pub mod ipc;

pub use manager::{SpoofIdentity, SpoofManager, ManagerError};
pub use ipc::{SpoofQuery, SpoofResponse};

// Global singleton (safe across threads)
use once_cell::sync::Lazy;
use parking_lot::RwLock;
static GLOBAL_MANAGER: Lazy<RwLock<Option<SpoofManager>>> = Lazy::new(|| RwLock::new(None));

/// Initialize global spoof manager (singleton pattern)
pub fn init_global(manager: SpoofManager) {
    *GLOBAL_MANAGER.write() = Some(manager);
}

/// Returns global spoof manager, or None if uninitialized.
pub fn global() -> Option<SpoofManager> {
    GLOBAL_MANAGER.read().clone()
}