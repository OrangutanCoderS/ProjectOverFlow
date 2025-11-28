use nix::libc;
use crate::errors::ManagerError;
use super::SuspendController;

/// macOS-safe controller for suspending and resuming processes.
pub struct MacOSSuspend;

impl SuspendController for MacOSSuspend {
    fn suspend(pid: i32) -> Result<(), ManagerError> {
        let res = unsafe { libc::kill(pid, libc::SIGSTOP) };
        if res == 0 { Ok(()) } else { Err(ManagerError::SignalFailed(pid, "SIGSTOP failed".into())) }
    }

    fn resume(pid: i32) -> Result<(), ManagerError> {
        let res = unsafe { libc::kill(pid, libc::SIGCONT) };
        if res == 0 { Ok(()) } else { Err(ManagerError::SignalFailed(pid, "SIGCONT failed".into())) }
    }

    fn exists(pid: i32) -> bool {
        unsafe { libc::kill(pid, 0) == 0 }
    }
}