use criterion::{criterion_group, criterion_main, Criterion};
use plugin_graph_runtime::*;
use serde_json;

const SIMPLE: &str = r#"
{
  "edges": {
    "A": ["B"],
    "B": ["C"],
    "C": []
  }
}
"#;

fn bench_load(c: &mut Criterion) {
    c.bench_function("plugin_graph_load", |b| {
        b.iter(|| {
            let def: PluginGraphDef = serde_json::from_str(SIMPLE).unwrap();
            let _ = PluginGraphRuntime::load(def).unwrap();
        });
    });
}

criterion_group!(benches, bench_load);
criterion_main!(benches);