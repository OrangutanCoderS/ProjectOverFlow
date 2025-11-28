//! Dispatch platform-safe syscall blockers

#[cfg(all(target_os = "macos", not(test)))]
pub use self::macos_safe::MacOSBlocker as PlatformSysBlocker;

#[cfg(all(target_os = "linux", not(test)))]
pub use self::linux_safe::LinuxBlocker as PlatformSysBlocker;

#[cfg(test)]
pub use self::mock::MockBlocker as PlatformSysBlocker;

pub mod linux_safe;
pub mod macos_safe;
pub mod mock;