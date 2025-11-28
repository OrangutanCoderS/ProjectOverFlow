//! Platform dispatcher for FakeOutput controllers.

#[cfg(all(target_os = "macos", not(test)))]
pub use self::macos_safe::MacOSFakeOutput as PlatformFakeOutput;

#[cfg(all(target_os = "linux", not(test)))]
pub use self::linux_safe::LinuxFakeOutput as PlatformFakeOutput;

// Tests always use the mock regardless of OS
#[cfg(test)]
pub use self::mock::MockFakeOutput as PlatformFakeOutput;

pub mod macos_safe;
pub mod linux_safe;
pub mod mock;