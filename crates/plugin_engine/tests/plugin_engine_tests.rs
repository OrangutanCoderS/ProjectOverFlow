use plugin_engine::{load_plugin, run_plugin, PluginMeta};
use std::path::PathBuf;

#[test]
fn test_plugin_load_and_run() {
    let path = PathBuf::from("plugins/rust/minimal_plugin/target/release/libminimal_plugin.dylib");

    let meta = load_plugin(path).expect("Plugin load failed");
    assert_eq!(meta.abi, 1);

    let result = run_plugin(&meta.name, "input string").expect("Plugin run failed");
    assert_eq!(result, 0);
}