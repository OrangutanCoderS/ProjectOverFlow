use crate::controller::*;
use crate::errors::ManagerError;
use crate::request::{ThrottleMode, ThrottleRequest};
use crate::audit::write_audit_log;
use std::thread;
use std::time::Duration;

pub struct ActionThrottleManager;

impl ActionThrottleManager {
    pub fn execute(req: ThrottleRequest) -> Result<(), ManagerError> {
        #[cfg(target_os = "macos")]
        type Ctrl = MacOSThrottle;
        #[cfg(target_os = "linux")]
        type Ctrl = LinuxThrottle;
        #[cfg(not(any(target_os = "macos", target_os = "linux")))]
        type Ctrl = MockThrottle;

        let pid = req.pid;
        let ctx = req.context.unwrap_or_else(|| "no_context".into());

        let mode_str = match req.mode {
            ThrottleMode::CPU => "CPU",
            ThrottleMode::IO => "IO",
            ThrottleMode::NET => "NET",
        };

        let result = match req.mode {
            ThrottleMode::CPU => Ctrl::limit_cpu(pid, req.intensity),
            ThrottleMode::IO  => Ctrl::limit_io(pid, req.intensity),
            ThrottleMode::NET => Ctrl::limit_net(pid, req.intensity),
        };

        match result {
            Ok(_) => {
                write_audit_log(pid, mode_str, req.intensity, req.duration_secs, false, &ctx)
                    .map_err(|e| ManagerError::Audit(e))?;

                // Spawn auto-restore thread
                if let Some(dur) = req.duration_secs {
                    let pid_restore = pid;
                    thread::spawn(move || {
                        thread::sleep(Duration::from_secs(dur));
                        let _ = Ctrl::restore(pid_restore);
                        let _ = write_audit_log(
                            pid_restore,
                            mode_str,
                            0.0,
                            None,
                            true,
                            "auto_restore",
                        );
                    });
                }

                Ok(())
            }
            Err(e) => Err(ManagerError::OperationFailed(e)),
        }
    }
}