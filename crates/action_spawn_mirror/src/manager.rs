use crate::controller::MirrorController;
use crate::{
    audit::write_audit_log,
    errors::MirrorError,
    request::MirrorRequest,
};
#[cfg(target_os = "macos")]
use crate::controller::macos_safe::MacOSMirror as Ctrl;
#[cfg(target_os = "linux")]
use crate::controller::linux_safe::LinuxMirror as Ctrl;
/// Local sandbox stub (self-contained)
struct Sandbox;

impl Sandbox {
    fn new() -> Result<Self, String> { Ok(Self) }

    fn apply_isolation(&self, pid: u32) -> Result<(), String> {
        println!("[sandbox] simulated isolation for pid {}", pid);
        Ok(())
    }
}

pub struct ActionSpawnMirror;

impl Default for ActionSpawnMirror {
    fn default() -> Self { Self }
}

impl ActionSpawnMirror {
    pub fn execute(&self, req: MirrorRequest) -> Result<(), MirrorError> {
        let (cmd, args) = Ctrl::snapshot(req.pid)
            .map_err(|e| MirrorError::SnapshotFailed(e))?;

        let mut result = "success";
        let mut mirror_pid: Option<u32> = None;

        let spawn_res = Ctrl::spawn_clone(&cmd, &args);
        match spawn_res {
            Ok(pid) => {
                mirror_pid = Some(pid);
                if req.sandboxed {
                    let sb = Sandbox::new()
                        .map_err(|e| MirrorError::SandboxFailed(format!("{e:?}")))?;
                    sb.apply_isolation(pid)
                        .map_err(|e| MirrorError::SandboxFailed(format!("{e:?}")))?;
                }
            }
            Err(e) => {
                result = "spawn_failed";
                return Err(MirrorError::SpawnFailed(e));
            }
        }

        write_audit_log(req.pid, mirror_pid, req.sandboxed, req.context.as_deref().unwrap_or("none"), result)
            .map_err(MirrorError::Audit)?;

        Ok(())
    }
}