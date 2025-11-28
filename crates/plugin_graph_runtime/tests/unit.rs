use plugin_graph_runtime::*;
use serde_json;

#[test]
fn test_cycle_detection() {
    let json = r#"
    {
        "edges": {
            "A": ["B"],
            "B": ["A"]
        }
    }
    "#;

    let def: PluginGraphDef = serde_json::from_str(json).unwrap();
    let result = PluginGraphRuntime::load(def);
    assert!(matches!(result, Err(PluginGraphError::CycleDetected)));
}