pub mod device;
pub mod backend;
pub mod emulator;

pub use device::{DeviceSpec, DeviceKind, Device, DeviceError, DeviceId};
pub use backend::{Backend, BackendError};
pub use backend::mock::MockBackend;
pub use emulator::{FakeDeviceManager, ManagerConfig, ManagerError};