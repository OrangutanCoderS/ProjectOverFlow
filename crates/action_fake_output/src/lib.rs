//! action_fake_output — safely injects or simulates fake outputs for observed processes.
//! Built for OverFlow Phase II – The Hand.
//! Fully user-space, reversible, logged, and SIP-compliant.

pub mod audit;
pub mod errors;
pub mod manager;
pub mod request;
pub mod controller;

use crate::errors::FakeOutputError;
/// Cross-platform interface defining fake output actions.
pub trait FakeOutputController {
    fn inject_stdout(pid: i32, data: &str) -> Result<(), FakeOutputError>;
    fn inject_stderr(pid: i32, data: &str) -> Result<(), FakeOutputError>;
    fn fake_file_read(path: &str, fake_data: &str) -> Result<(), FakeOutputError>;
    fn fake_command_response(command: &str, fake_output: &str) -> Result<(), FakeOutputError>;
    fn restore(pid: i32) -> Result<(), FakeOutputError>;
}