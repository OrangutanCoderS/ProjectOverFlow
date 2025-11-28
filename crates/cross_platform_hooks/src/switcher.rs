use std::path::{Path, PathBuf};

use tracing::info;

use crate::error::ConfigError;
use crate::linux::LinuxPortConfigLoaderFs;
use crate::macos::MacOsHooksConfig;
use crate::model::{LinuxPortConfig, PlatformKind, WindowsHooksConfig};
use crate::windows::WindowsHooksLoaderFs;

/// Aggregated view of platform-specific config.
#[derive(Debug, Clone)]
pub struct CrossPlatformConfig {
    platform: PlatformKind,
    linux_ports: Option<LinuxPortConfig>,
    windows_hooks: Option<WindowsHooksConfig>,
    macos_hooks: Option<MacOsHooksConfig>,
}

impl CrossPlatformConfig {
    pub fn platform(&self) -> PlatformKind {
        self.platform
    }

    pub fn linux_ports(&self) -> Option<&LinuxPortConfig> {
        self.linux_ports.as_ref()
    }

    pub fn windows_hooks(&self) -> Option<&WindowsHooksConfig> {
        self.windows_hooks.as_ref()
    }

    pub fn macos_hooks(&self) -> Option<&MacOsHooksConfig> {
        self.macos_hooks.as_ref()
    }

    /// High-level entry: detect current platform and load config
    /// from a directory containing YAMLs.
    ///
    /// Expected files:
    /// - linux:   `<root>/linux_port_config.yaml`
    /// - windows: `<root>/windows_hooks.yaml`
    /// - macos:   none required (currently)
    pub fn detect_and_load<P: AsRef<Path>>(config_root: P) -> Result<Self, ConfigError> {
        let platform = PlatformKind::detect();
        Self::load_for(platform, config_root)
    }

    /// Testable variant: caller can inject explicit platform kind.
    pub fn load_for<P: AsRef<Path>>(
        platform: PlatformKind,
        config_root: P,
    ) -> Result<Self, ConfigError> {
        let root = config_root.as_ref();
        info!("CrossPlatformConfig::load_for platform={:?}", platform);

        match platform {
            PlatformKind::Linux => {
                let linux_path = root.join("linux_port_config.yaml");
                let loader = LinuxPortConfigLoaderFs::new(linux_path);
                let linux_ports = loader.load()?;
                Ok(Self {
                    platform,
                    linux_ports: Some(linux_ports),
                    windows_hooks: None,
                    macos_hooks: None,
                })
            }
            PlatformKind::Windows => {
                let win_path = root.join("windows_hooks.yaml");
                let loader = WindowsHooksLoaderFs::new(win_path);
                let windows_hooks = loader.load()?;
                Ok(Self {
                    platform,
                    linux_ports: None,
                    windows_hooks: Some(windows_hooks),
                    macos_hooks: None,
                })
            }
            PlatformKind::MacOs => {
                // Currently macOS doesn't require extra YAML config.
                let mac = MacOsHooksConfig::new_empty();
                mac.validate()?;
                Ok(Self {
                    platform,
                    linux_ports: None,
                    windows_hooks: None,
                    macos_hooks: Some(mac),
                })
            }
            PlatformKind::Unknown => Err(ConfigError::Unsupported(PlatformKind::Unknown)),
        }
    }
}
