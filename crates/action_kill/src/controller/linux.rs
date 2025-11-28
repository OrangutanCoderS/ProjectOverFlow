use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;
use crate::errors::ManagerError;
use super::ProcessController;

pub struct LinuxProcController;

impl LinuxProcController {
    pub fn new() -> Self { Self }
}

impl ProcessController for LinuxProcController {
    fn exists(&self, pid: i32) -> bool {
        nix::unistd::Pid::from_raw(pid).as_raw() > 0
    }

    fn send_signal(&self, pid: i32, signal: i32) -> Result<(), ManagerError> {
        let pid = Pid::from_raw(pid);
        let sig = Signal::try_from(signal)
            .map_err(|_| ManagerError::Other(format!("invalid signal {signal}")))?;
        kill(pid, Some(sig)).map_err(ManagerError::Backend)
    }
}