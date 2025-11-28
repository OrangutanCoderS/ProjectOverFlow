#![forbid(unsafe_code)]
use std::ops::Range;
use std::time::Duration;
use crate::error::PatchError;

#[derive(Debug, Clone)]
pub struct MemoryRegion {
    pub name: String,          // e.g., ".text" or "anon"
    pub range: Range<usize>,   // start..end (end exclusive)
    pub prot: String,          // "r-x", "rw-"
}

pub trait OsProcessMem: Send + Sync {
    fn check_permission(&self, pid: i32) -> Result<(), PatchError>;
    fn list_regions(&self, pid: i32) -> Result<Vec<MemoryRegion>, PatchError>;
    fn read(&self, pid: i32, addr: usize, buf: &mut [u8], timeout: Duration) -> Result<(), PatchError>;
    fn write(&self, pid: i32, addr: usize, data: &[u8], timeout: Duration) -> Result<(), PatchError>;
}

#[cfg(feature = "mock")]
pub mod mock;

#[cfg(feature = "macos")]
pub mod macos; // to be added later (mach_vm_*)

#[cfg(feature = "linux")]
pub mod linux; // to be added later (process_vm_*)

/// Factory for the active backend
pub fn backend() -> Box<dyn OsProcessMem> {
    #[cfg(feature = "mock")]
    { Box::new(mock::MockBackend::new()) }

    #[cfg(all(not(feature="mock"), feature="macos", not(feature="linux")))]
    { Box::new(macos::MacBackend::new()) }

    #[cfg(all(not(feature="mock"), feature="linux", not(feature="macos")))]
    { Box::new(linux::LinuxBackend::new()) }

    #[cfg(all(not(feature="mock"), not(feature="macos"), not(feature="linux")))]
    { compile_error!("No backend enabled. Use feature 'mock', 'macos', or 'linux'."); }
}