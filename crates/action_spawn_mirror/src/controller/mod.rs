#[cfg(target_os = "macos")]
pub mod macos_safe;
#[cfg(target_os = "linux")]
pub mod linux_safe;
pub mod mock;

pub trait MirrorController {
    fn snapshot(pid: i32) -> Result<(String, Vec<String>), String>;
    fn spawn_clone(cmd: &str, args: &[String]) -> Result<u32, String>;
}