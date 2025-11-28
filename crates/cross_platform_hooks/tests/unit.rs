use std::fs;

use cross_platform_hooks::{
    linux::LinuxPortConfigLoaderFs,
    switcher::CrossPlatformConfig,
    ConfigError,
    DefaultPolicy,
    LinuxPortConfig,
    LinuxPortRule,
    PlatformKind,
    Protocol,
    WindowsHooksConfig,
    WindowsProviderConfig,
};
use tempfile::tempdir;

const SAMPLE_LINUX_YAML: &str = r#"
default_policy: deny
rules:
  - port: 22
    protocol: tcp
    tag: "ssh"
    allow: true
  - port: 8080
    protocol: tcp
    tag: "overflow-http"
    allow: false
"#;

const SAMPLE_WINDOWS_YAML: &str = r#"
providers:
  - name: "Microsoft-Windows-Kernel-Process"
    level: "Informational"
    keywords: ["ProcessStart", "ProcessStop"]
wmi_queries:
  - "SELECT * FROM Win32_Process"
"#;

#[test]
fn linux_config_validation_allows_valid_rules() {
    let cfg = LinuxPortConfig {
        rules: vec![
            LinuxPortRule {
                port: 80,
                protocol: Protocol::Tcp,
                tag: "http".to_string(),
                allow: true,
                note: None,
            },
            LinuxPortRule {
                port: 53,
                protocol: Protocol::Udp,
                tag: "dns".to_string(),
                allow: true,
                note: Some("DNS".into()),
            },
        ],
        default_policy: DefaultPolicy::Deny,
    };

    assert!(cfg.validate().is_ok());
}

#[test]
fn linux_config_rejects_invalid_port_zero() {
    let cfg = LinuxPortConfig {
        rules: vec![LinuxPortRule {
            port: 0,
            protocol: Protocol::Tcp,
            tag: "bad".to_string(),
            allow: true,
            note: None,
        }],
        default_policy: DefaultPolicy::Allow,
    };

    let err = cfg.validate().unwrap_err();
    match err {
        ConfigError::Invalid(msg) => assert!(msg.contains("port 0")),
        _ => panic!("expected Invalid error"),
    }
}

#[test]
fn linux_loader_reads_yaml_from_disk() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("linux_port_config.yaml");
    fs::write(&path, SAMPLE_LINUX_YAML).unwrap();

    let loader = LinuxPortConfigLoaderFs::new(&path);
    let cfg = loader.load().unwrap();

    assert_eq!(cfg.rules.len(), 2);
    assert_eq!(cfg.default_policy, DefaultPolicy::Deny);
    assert_eq!(cfg.rules[0].port, 22);
}

#[test]
fn windows_config_validation_rejects_empty_provider_name() {
    let cfg = WindowsHooksConfig {
        providers: vec![WindowsProviderConfig {
            name: "".to_string(),
            guid: None,
            level: "Verbose".to_string(),
            keywords: vec![],
        }],
        wmi_queries: vec![],
    };

    let err = cfg.validate().unwrap_err();
    match err {
        ConfigError::Invalid(msg) => assert!(msg.contains("provider name")),
        _ => panic!("expected Invalid error"),
    }
}

#[test]
fn switcher_loads_linux_when_requested() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("linux_port_config.yaml"),
        SAMPLE_LINUX_YAML,
    )
    .unwrap();

    let cfg = CrossPlatformConfig::load_for(PlatformKind::Linux, dir.path()).unwrap();
    assert_eq!(cfg.platform(), PlatformKind::Linux);
    assert!(cfg.linux_ports().is_some());
    assert!(cfg.windows_hooks().is_none());
}

#[test]
fn switcher_loads_windows_when_requested() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("windows_hooks.yaml"),
        SAMPLE_WINDOWS_YAML,
    )
    .unwrap();

    let cfg =
        CrossPlatformConfig::load_for(PlatformKind::Windows, dir.path()).unwrap();
    assert_eq!(cfg.platform(), PlatformKind::Windows);
    assert!(cfg.windows_hooks().is_some());
    assert!(cfg.linux_ports().is_none());
}

#[test]
fn switcher_mac_os_has_empty_config_but_supported() {
    let dir = tempdir().unwrap();
    let cfg =
        CrossPlatformConfig::load_for(PlatformKind::MacOs, dir.path()).unwrap();

    assert_eq!(cfg.platform(), PlatformKind::MacOs);
    assert!(cfg.macos_hooks().is_some());
    assert!(cfg.linux_ports().is_none());
    assert!(cfg.windows_hooks().is_none());
}

#[test]
fn switcher_unknown_platform_is_rejected() {
    let dir = tempdir().unwrap();
    let err =
        CrossPlatformConfig::load_for(PlatformKind::Unknown, dir.path()).unwrap_err();
    match err {
        ConfigError::Unsupported(p) => assert_eq!(p, PlatformKind::Unknown),
        _ => panic!("expected Unsupported error"),
    }
}
