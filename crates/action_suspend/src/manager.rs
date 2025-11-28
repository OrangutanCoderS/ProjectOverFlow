#[cfg(target_os = "macos")]
use crate::controller::macos_safe::MacOSSuspend;
#[cfg(target_os = "linux")]
use crate::controller::linux_safe::LinuxSuspend;

use crate::{
    audit::write_audit_log,
    controller::SuspendController,
    errors::ManagerError,
    request::{SuspendMode, SuspendRequest},
};

/// High-level suspend/resume manager handling validation,
/// audit logging, and platform abstraction.
///
/// This layer must never directly invoke unsafe syscalls.
/// OS-specific unsafe sections are encapsulated in `controller/*_safe.rs`.
pub struct ActionSuspendManager;

impl Default for ActionSuspendManager {
    fn default() -> Self {
        Self
    }
}

impl ActionSuspendManager {
    /// Executes a suspend or resume operation safely.
    ///
    /// Performs PID validation, existence checks, and delegates
    /// to the platform-specific controller (MacOSSuspend or LinuxSuspend).
    pub fn execute(&self, req: SuspendRequest) -> Result<(), ManagerError> {
        let pid = req.pid;
        let ctx = req.context.unwrap_or_else(|| "no_context".into());

        // --- ✅ PID VALIDATION GUARD ---
        // Disallow dangerous PIDs that could hang or kill system-wide sessions.
        // pid <= 0 -> reserved for broadcast or system calls.
        // pid == 1 -> launchd/systemd/root daemon.
        if pid <= 1 {
            return Err(ManagerError::InvalidPid(pid));
        }

        // --- ✅ SELECT CONTROLLER BASED ON OS ---
        #[cfg(target_os = "macos")]
        type Ctrl = MacOSSuspend;
        #[cfg(target_os = "linux")]
        type Ctrl = LinuxSuspend;

        // --- ✅ PROCESS EXISTENCE CHECK ---
        if !Ctrl::exists(pid) {
            return Err(ManagerError::InvalidPid(pid));
        }

        // --- ✅ DELEGATE ACTION ---
        let result = match req.mode {
            SuspendMode::Suspend => Ctrl::suspend(pid),
            SuspendMode::Resume => Ctrl::resume(pid),
        };

        // --- ✅ AUDIT + ERROR PROPAGATION ---
        match result {
            Ok(_) => write_audit_log(pid, &format!("{:?}", req.mode), &ctx),
            Err(e) => Err(e),
        }
    }
}