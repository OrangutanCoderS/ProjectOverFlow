//! MockFakeOutput — test-only implementation of the FakeOutputController trait.
//! It performs no actual I/O redirection; it just simulates fake output and logs calls.

use crate::errors::FakeOutputError;
use crate::audit::log_event;
use crate::FakeOutputController;

/// Mock implementation of FakeOutputController, used for unit testing and safe simulation.
pub struct MockFakeOutput;

impl FakeOutputController for MockFakeOutput {
    fn inject_stdout(pid: i32, data: &str) -> Result<(), FakeOutputError> {
        println!("[mock] Inject stdout to pid={pid}: {data}");
        log_event(Some(pid), "inject_stdout(mock)", "stdout", Some(data), "ok");
        Ok(())
    }

    fn inject_stderr(pid: i32, data: &str) -> Result<(), FakeOutputError> {
        println!("[mock] Inject stderr to pid={pid}: {data}");
        log_event(Some(pid), "inject_stderr(mock)", "stderr", Some(data), "ok");
        Ok(())
    }

    fn fake_file_read(path: &str, fake_data: &str) -> Result<(), FakeOutputError> {
        println!("[mock] Fake file read for path={path}: {fake_data}");
        log_event(None, "fake_file_read(mock)", path, Some(fake_data), "ok");
        Ok(())
    }

    fn fake_command_response(command: &str, fake_output: &str) -> Result<(), FakeOutputError> {
        println!("[mock] Fake command response: {command} -> {fake_output}");
        log_event(None, "fake_command_response(mock)", command, Some(fake_output), "ok");
        Ok(())
    }

    fn restore(pid: i32) -> Result<(), FakeOutputError> {
        println!("[mock] Restore pid={pid}");
        log_event(Some(pid), "restore(mock)", "stdout/stderr", None, "ok");
        Ok(())
    }
}