//! Device spec definitions and validation.
// Keep types small and serializable.

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use thiserror::Error;

/// Common error type for device spec validation.
#[derive(Debug, Error)]
pub enum DeviceError {
    #[error("invalid parameter: {0}")]
    InvalidParam(String),
}

/// Unique device identifier
pub type DeviceId = Uuid;

/// Supported simulated device kinds.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DeviceKind {
    HidKeyboard,
    HidMouse,
    BlockDevice,     // simulated block device (size-only)
    VirtualNetIface, // simulated NIC
}

/// DeviceSpec: describes a device to create.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceSpec {
    pub kind: DeviceKind,
    pub vendor: Option<String>,
    pub product: Option<String>,
    #[serde(default)]
    pub metadata: serde_json::Value,
    /// For BlockDevice: size in bytes (optional for others)
    pub size_bytes: Option<u64>,
}

impl DeviceSpec {
    /// Validate semantics for this spec.
    pub fn validate(&self) -> Result<(), DeviceError> {
        match self.kind {
            DeviceKind::BlockDevice => {
                let size = self.size_bytes.unwrap_or(0);
                if size == 0 {
                    return Err(DeviceError::InvalidParam(
                        "block device must have size_bytes > 0".into(),
                    ));
                }
            }
            DeviceKind::VirtualNetIface => {
                // for future extendability, require metadata.mac if present to be string
                if !self.metadata.is_null() {
                    // no strict check here — leave flexible
                }
            }
            _ => {}
        }
        Ok(())
    }
}

/// A handle for an instantiated, simulated device.
#[derive(Debug, Clone)]
pub struct Device {
    pub id: DeviceId,
    pub spec: DeviceSpec,
    pub created_at: DateTime<Utc>,
    pub active: bool,
}

impl Device {
    pub fn new(spec: DeviceSpec) -> Self {
        Self {
            id: DeviceId::new_v4(),
            spec,
            created_at: chrono::Utc::now(),
            active: true,
        }
    }

    pub fn deactivate(&mut self) {
        self.active = false;
    }

    pub fn activate(&mut self) {
        self.active = true;
    }
}