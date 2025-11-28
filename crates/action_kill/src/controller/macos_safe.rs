#![allow(unsafe_code)]
use crate::errors::ManagerError;
use super::ProcessController;
use nix::libc;

pub struct MacProcController;

impl MacProcController {
    pub fn new() -> Self { Self }

    #[inline(always)]
    fn safe_kill(pid: i32, signal: i32) -> Result<(), ManagerError> {
        // Safe wrapper around libc::kill
        // This FFI boundary is minimal and strictly audited.
        let result = unsafe { libc::kill(pid, signal) };
        if result == 0 {
            Ok(())
        } else {
            Err(ManagerError::Backend(nix::Error::last()))
        }
    }
}

impl ProcessController for MacProcController {
    fn exists(&self, pid: i32) -> bool {
        unsafe { libc::kill(pid, 0) == 0 }
    }

    fn send_signal(&self, pid: i32, signal: i32) -> Result<(), ManagerError> {
        Self::safe_kill(pid, signal)
    }
}