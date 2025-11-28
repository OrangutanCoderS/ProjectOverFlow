use criterion::{criterion_group, criterion_main, Criterion};
use plugin_graph_builder::builder::PluginGraphBuilder;
use plugin_graph_builder::builder::loader::PluginGraphDef;

const JSON: &str = r#"
{
    "fake_net": ["entropy_flagger"],
    "entropy_flagger": []
}
"#;

fn bench_build(c: &mut Criterion) {
    c.bench_function("plugin_graph_builder_build", |b| {
        b.iter(|| {
            let def = PluginGraphDef::load_from_str(JSON).unwrap();
            let g = PluginGraphBuilder::build(def).unwrap();
            assert!(g.layers.len() == 2);
        })
    });
}

criterion_group!(benches, bench_build);
criterion_main!(benches);
