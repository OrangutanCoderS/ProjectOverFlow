use crate::errors::FakeOutputError;
use crate::audit::log_event;
use crate::FakeOutputController;
use std::fs::OpenOptions;
use std::io::Write;

/// macOS-safe fake output controller — user-space simulation only.
pub struct MacOSFakeOutput;

impl FakeOutputController for MacOSFakeOutput {
    fn inject_stdout(pid: i32, data: &str) -> Result<(), FakeOutputError> {
        let path = format!("/tmp/fake_stdout_{}.log", pid);
        if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(&path) {
            f.write_all(data.as_bytes()).map_err(|e| FakeOutputError::Io(e.to_string()))?;
            log_event(Some(pid), "inject_stdout", &path, Some(data), "success");
            Ok(())
        } else {
            Err(FakeOutputError::PermissionDenied)
        }
    }

    fn inject_stderr(pid: i32, data: &str) -> Result<(), FakeOutputError> {
        let path = format!("/tmp/fake_stderr_{}.log", pid);
        if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(&path) {
            f.write_all(data.as_bytes()).map_err(|e| FakeOutputError::Io(e.to_string()))?;
            log_event(Some(pid), "inject_stderr", &path, Some(data), "success");
            Ok(())
        } else {
            Err(FakeOutputError::PermissionDenied)
        }
    }

    fn fake_file_read(path: &str, fake_data: &str) -> Result<(), FakeOutputError> {
        let tmp = format!("{}.fake", path);
        if let Ok(mut f) = OpenOptions::new().create(true).truncate(true).open(&tmp) {
            f.write_all(fake_data.as_bytes()).map_err(|e| FakeOutputError::Io(e.to_string()))?;
            log_event(None, "fake_file_read", path, Some(fake_data), "success");
            Ok(())
        } else {
            Err(FakeOutputError::InvalidPath(path.into()))
        }
    }

    fn fake_command_response(command: &str, fake_output: &str) -> Result<(), FakeOutputError> {
        println!("[fake-output] {} → {}", command, fake_output);
        log_event(None, "fake_command_response", command, Some(fake_output), "success");
        Ok(())
    }

    fn restore(pid: i32) -> Result<(), FakeOutputError> {
        log_event(Some(pid), "restore", "stdout/stderr", None, "success");
        Ok(())
    }
}