//! Backend trait and re-exports.

use crate::device::{Device, DeviceSpec, DeviceId};
use thiserror::Error;
pub mod mock;

/// Backend error types for backend implementations.
#[derive(Debug, Error)]
pub enum BackendError {
    #[error("backend failure: {0}")]
    BackendFailure(String),
    #[error("not found")]
    NotFound,
}

/// Backend trait: manager uses this abstraction. MockBackend implements for testing.
pub trait Backend: Send + Sync + 'static {
    /// Create a device from a spec. Returns created Device.
    fn create_device(&self, spec: DeviceSpec) -> Result<Device, BackendError>;

    /// Remove device by id.
    fn remove_device(&self, id: &DeviceId) -> Result<(), BackendError>;

    /// Inject an input/event into device (serialized event).
    fn inject_event(&self, id: &DeviceId, event: serde_json::Value) -> Result<(), BackendError>;

    /// List devices currently managed by backend.
    fn list_devices(&self) -> Result<Vec<Device>, BackendError>;
}