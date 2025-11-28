use context_graph::{graph::ContextGraph, graph::GraphDef};

#[test]
fn load_basic_graph() {
    let data = r#"
{
  "nodes": [
    { "key": {"Name":"A"}, "meta":{"priority":1,"entropy":0.2,"sensitivity":2,"reactiveness":3} },
    { "key": {"Name":"B"}, "meta":{"priority":1,"entropy":0.3,"sensitivity":3,"reactiveness":4} }
  ],
  "edges": {
    "A": [
      {"to":1, "weight":1.0, "label":null, "conditions":null}
    ]
  }
}
"#;

    let def: GraphDef = serde_json::from_str(data).unwrap();
    let graph = ContextGraph::load(def).unwrap();

    assert_eq!(graph.nodes.len(), 2);
    assert_eq!(graph.outgoing[0].len(), 1);
}