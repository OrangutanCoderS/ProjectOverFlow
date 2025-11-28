//! Emulator manager: orchestrates devices using a Backend trait implementation.
// Thread-safe, minimal public surface.

use crate::device::{Device, DeviceSpec, DeviceId};
use crate::backend::{Backend, BackendError};
use parking_lot::RwLock;
use std::sync::Arc;
use chrono::Utc;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum ManagerError {
    #[error("validation error: {0}")]
    Validation(String),
    #[error("backend error: {0}")]
    Backend(String),
}

impl From<BackendError> for ManagerError {
    fn from(e: BackendError) -> Self {
        match e {
            BackendError::BackendFailure(s) => ManagerError::Backend(s),
            BackendError::NotFound => ManagerError::Backend("not found".into()),
        }
    }
}

/// ManagerConfig: extendable.
#[derive(Debug, Clone)]
pub struct ManagerConfig {
    pub max_devices: usize,
}

impl Default for ManagerConfig {
    fn default() -> Self {
        Self { max_devices: 256 }
    }
}

#[derive(Clone)]
pub struct FakeDeviceManager {
    backend: Arc<dyn Backend>,
    config: ManagerConfig,
    // local index for fast access without asking backend for everything.
    index: Arc<RwLock<std::collections::HashMap<DeviceId, Device>>>,
}

impl FakeDeviceManager {
    /// Create with any backend (for production we'll add os backend behind feature flag).
    pub fn new(backend: Arc<dyn Backend>, config: ManagerConfig) -> Self {
        Self {
            backend,
            config,
            index: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Register a device from a spec. Returns Device on success.
    pub fn register_device(&self, spec: DeviceSpec) -> Result<Device, ManagerError> {
        spec.validate().map_err(|e| ManagerError::Validation(e.to_string()))?;

        // capacity check
        if self.index.read().len() >= self.config.max_devices {
            return Err(ManagerError::Validation("device limit reached".into()));
        }

        let dev = self.backend.create_device(spec).map_err(ManagerError::from)?;
        let id = dev.id;
        self.index.write().insert(id, dev.clone());
        Ok(dev)
    }

    /// Unregister / remove device
    pub fn unregister_device(&self, id: &DeviceId) -> Result<(), ManagerError> {
        self.backend.remove_device(id).map_err(ManagerError::from)?;
        self.index.write().remove(id);
        Ok(())
    }

    /// Inject an event into a device (serialized JSON event).
    pub fn inject(&self, id: &DeviceId, event: serde_json::Value) -> Result<(), ManagerError> {
        self.backend.inject_event(id, event).map_err(ManagerError::from)
    }

    /// List devices (snapshot).
    pub fn list(&self) -> Vec<Device> {
        self.index.read().values().cloned().collect()
    }

    /// Clear state (useful for deterministic shutdown in tests).
    pub fn cleanup(&self) {
        let ids: Vec<DeviceId> = self.index.read().keys().cloned().collect();
        for id in ids {
            // best-effort: ignore backend errors during cleanup
            let _ = self.backend.remove_device(&id);
            self.index.write().remove(&id);
        }
    }
}