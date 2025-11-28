#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "macos")]
pub mod macos_safe;
pub mod mock;

use crate::errors::ManagerError;

pub trait ProcessController: Send + Sync {
    fn exists(&self, pid: i32) -> bool;
    fn send_signal(&self, pid: i32, signal: i32) -> Result<(), ManagerError>;
}