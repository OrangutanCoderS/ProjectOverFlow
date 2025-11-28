use plugin_registry::{PluginMetadata, PluginRegistry};

#[test]
fn register_and_get_plugin() {
    let mut registry = PluginRegistry::new();

    let meta = PluginMetadata::new("netmon", "Network Monitor");
    registry.register(meta.clone()).expect("register should succeed");

    assert!(registry.is_registered("netmon"));
    let stored = registry.get("netmon").expect("plugin must be present");
    assert_eq!(stored.id, "netmon");
    assert_eq!(stored.name, "Network Monitor");
}

#[test]
fn duplicate_registration_fails() {
    let mut registry = PluginRegistry::new();

    let meta = PluginMetadata::new("filemon", "File Monitor");
    registry.register(meta.clone()).expect("first register ok");

    let err = registry
        .register(meta)
        .expect_err("second register must fail");

    let msg = format!("{err}");
    assert!(
        msg.contains("already registered"),
        "error message should mention duplicate"
    );
}

#[test]
fn unregister_removes_plugin() {
    let mut registry = PluginRegistry::new();

    let meta = PluginMetadata::new("periphmon", "Peripheral Monitor");
    registry.register(meta).expect("register ok");

    let removed = registry
        .unregister("periphmon")
        .expect("unregister must succeed");

    assert_eq!(removed.id, "periphmon");
    assert!(!registry.is_registered("periphmon"));
    assert!(registry.get("periphmon").is_none());
}

#[test]
fn unregister_nonexistent_fails() {
    let mut registry = PluginRegistry::new();

    let err = registry
        .unregister("does_not_exist")
        .expect_err("unregistering unknown plugin should fail");

    let msg = format!("{err}");
    assert!(msg.contains("not registered"));
}