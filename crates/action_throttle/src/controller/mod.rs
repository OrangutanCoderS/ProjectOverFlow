// Declare submodules first
#[cfg(target_os = "macos")]
mod macos_safe;
#[cfg(target_os = "linux")]
mod linux_safe;
#[cfg(not(any(target_os = "macos", target_os = "linux")))]
mod mock;

// Re-export platform implementations
#[cfg(target_os = "macos")]
pub use macos_safe::MacOSThrottle;
#[cfg(target_os = "linux")]
pub use linux_safe::LinuxThrottle;
#[cfg(not(any(target_os = "macos", target_os = "linux")))]
pub use mock::MockThrottle;

// Trait shared by all throttle controllers
pub trait ThrottleController {
    fn limit_cpu(pid: i32, intensity: f32) -> Result<(), String>;
    fn limit_io(pid: i32, intensity: f32) -> Result<(), String>;
    fn limit_net(pid: i32, intensity: f32) -> Result<(), String>;
    fn restore(pid: i32) -> Result<(), String>;
}