//! Mocked syscall blocker for safe unit tests.

use crate::audit::log_event;
use crate::errors::SysBlockError;

pub struct MockBlocker;

impl MockBlocker {
    pub fn block_syscall(pid: i32, syscall: &str, mode: &str) -> Result<(), SysBlockError> {
        log_event(pid, syscall, mode, "mock", "simulated_block");
        Ok(())
    }
}