//! Safe syscall blocking for Linux (ptrace + EPERM simulation)

use crate::errors::SysBlockError;
use crate::audit::log_event;
use std::process::Command;
#[cfg(target_os = "linux")]
use nix::unistd::Uid;


pub struct LinuxBlocker;

impl LinuxBlocker {
    pub fn block_syscall(pid: i32, syscall: &str, mode: &str) -> Result<(), SysBlockError> {
        if !nix::unistd::Uid::effective().is_root() {
            return Err(SysBlockError::PermissionDenied);
        }

        let allow_list = ["read", "write", "close", "exit"];
        if allow_list.contains(&syscall) {
            return Err(SysBlockError::SyscallDenied(syscall.to_string()));
        }

        let result = Command::new("sudo")
            .arg("strace")
            .arg("-p")
            .arg(pid.to_string())
            .arg("-e")
            .arg(format!("trace={}", syscall))
            .output();

        match result {
            Ok(_) => {
                log_event(pid, syscall, mode, "Linux", "blocked");
                Ok(())
            }
            Err(e) => Err(SysBlockError::IoError(e.to_string())),
        }
    }
}