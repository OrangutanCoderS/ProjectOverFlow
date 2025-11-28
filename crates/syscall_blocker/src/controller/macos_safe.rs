//! Safe syscall blocker for macOS using DTrace probes

use crate::audit::log_event;
use crate::errors::SysBlockError;
use std::fs;
use std::process::Command;
use nix::libc;
pub struct MacOSBlocker;

impl MacOSBlocker {
    pub fn block_syscall(pid: i32, syscall: &str, mode: &str) -> Result<(), SysBlockError> {
        if unsafe { libc::geteuid() } != 0 {
            return Err(SysBlockError::PermissionDenied);
        }

        let dtrace_script = format!(
            "syscall::{syscall}:entry /pid == {pid}/ {{ printf(\"Blocked syscall: {syscall}\\n\"); raise(SIGSTOP); }}"
        );
        let path = "/tmp/block_syscall.d";
        fs::write(path, &dtrace_script)
            .map_err(|e| SysBlockError::IoError(e.to_string()))?;

        let result = Command::new("sudo")
            .arg("dtrace")
            .arg("-q")
            .arg("-s")
            .arg(path)
            .output();

        match result {
            Ok(_) => {
                log_event(pid, syscall, mode, "macOS", "blocked");
                Ok(())
            }
            Err(e) => Err(SysBlockError::IoError(e.to_string())),
        }
    }
}