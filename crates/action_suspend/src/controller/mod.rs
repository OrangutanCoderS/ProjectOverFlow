#[cfg(target_os = "macos")]
pub mod macos_safe;
#[cfg(target_os = "linux")]
pub mod linux_safe;

use crate::errors::ManagerError;

/// Common trait interface for suspend/resume across OSes.
pub trait SuspendController {
    fn suspend(pid: i32) -> Result<(), ManagerError>;
    fn resume(pid: i32) -> Result<(), ManagerError>;
    fn exists(pid: i32) -> bool;
}