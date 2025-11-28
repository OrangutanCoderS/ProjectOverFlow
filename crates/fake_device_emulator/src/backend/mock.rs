//! Simple in-memory mock backend used by default.
// Thread-safe, deterministic, and easily tested.
use super::Backend;
use crate::device::{Device, DeviceSpec, DeviceId};
use crate::backend::BackendError;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct MockBackend {
    inner: Arc<RwLock<HashMap<DeviceId, Device>>>,
}

impl MockBackend {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for MockBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl Backend for MockBackend {
    fn create_device(&self, spec: DeviceSpec) -> Result<Device, BackendError> {
        // Defensive: spec should be validated earlier, but double-check
        spec.validate().map_err(|e| BackendError::BackendFailure(e.to_string()))?;
        let mut dev = Device::new(spec);
        let id = dev.id;
        self.inner.write().insert(id, dev.clone());
        Ok(dev)
    }

    fn remove_device(&self, id: &DeviceId) -> Result<(), BackendError> {
        let removed = self.inner.write().remove(id);
        removed.map(|_| ()).ok_or(BackendError::NotFound)
    }

    fn inject_event(&self, id: &DeviceId, _event: serde_json::Value) -> Result<(), BackendError> {
        // For mock: accept event if device exists and is active.
        let guard = self.inner.read();
        match guard.get(id) {
            Some(dev) if dev.active => Ok(()),
            Some(_) => Err(BackendError::BackendFailure("device inactive".into())),
            None => Err(BackendError::NotFound),
        }
    }

    fn list_devices(&self) -> Result<Vec<Device>, BackendError> {
        Ok(self.inner.read().values().cloned().collect())
    }
}