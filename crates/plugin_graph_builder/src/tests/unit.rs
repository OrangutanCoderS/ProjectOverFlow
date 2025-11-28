use plugin_graph_builder::*;
use plugin_graph_builder::builder::PluginGraphBuilder;
use plugin_graph_builder::builder::loader::PluginGraphDef;

#[test]
fn test_basic_graph() {
    let json = r#"
    {
        "fake_net": ["entropy_flagger"],
        "entropy_flagger": []
    }
    "#;

    let def = PluginGraphDef::load_from_str(json).unwrap();
    let g = PluginGraphBuilder::build(def).unwrap();

    assert_eq!(g.layers.len(), 2);
    assert!(g.layers[0].contains(&"entropy_flagger".to_string()));
    assert!(g.layers[1].contains(&"fake_net".to_string()));
}
