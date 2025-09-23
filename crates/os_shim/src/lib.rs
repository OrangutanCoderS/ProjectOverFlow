#![deny(clippy::all, clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

/// OS-specific safe wrappers around syscalls.
/// Any unsafe lives here, tightly scoped and tested.

use libc;

/// Sends a signal to a process by PID. Safe wrapper around libc::kill.
///
/// Returns:
/// - Ok(()) if signal was sent successfully.
/// - Err(errno) if the syscall failed.
pub fn kill_process(pid: i32, sig: i32) -> Result<(), i32> {
    let rc = unsafe { libc::kill(pid, sig) };
    if rc == 0 {
        Ok(())
    } else {
        Err(errno())
    }
}

/// Get the last OS error code (errno).
fn errno() -> i32 {
    unsafe { *libc::__error() }
}
