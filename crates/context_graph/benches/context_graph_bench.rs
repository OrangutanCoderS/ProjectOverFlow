use criterion::{criterion_group, criterion_main, Criterion};
use context_graph::{graph::{ContextGraph, GraphDef}};

fn bench_load(c: &mut Criterion) {
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

    c.bench_function("load_basic_graph", |b| {
        b.iter(|| {
            let def: GraphDef = serde_json::from_str(data).unwrap();
            let _ = ContextGraph::load(def).unwrap();
        });
    });
}

criterion_group!(benches, bench_load);
criterion_main!(benches);