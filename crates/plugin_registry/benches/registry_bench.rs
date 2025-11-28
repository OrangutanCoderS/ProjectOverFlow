use criterion::{criterion_group, criterion_main, Criterion};

use plugin_registry::{PluginMetadata, PluginRegistry};

fn bench_register_many(c: &mut Criterion) {
    c.bench_function("registry_register_1k", |b| {
        b.iter(|| {
            let mut registry = PluginRegistry::new();
            for i in 0..1_000 {
                let id = format!("plugin_{i}");
                let meta = PluginMetadata::new(&id, &id);
                registry.register(meta).unwrap();
            }
            assert_eq!(registry.len(), 1_000);
        });
    });
}

fn bench_lookup(c: &mut Criterion) {
    let mut registry = PluginRegistry::new();
    for i in 0..1_000 {
        let id = format!("plugin_{i}");
        let meta = PluginMetadata::new(&id, &id);
        registry.register(meta).unwrap();
    }

    c.bench_function("registry_lookup_hot", |b| {
        b.iter(|| {
            for i in 0..1_000 {
                let id = format!("plugin_{i}");
                let meta = registry.get(&id).unwrap();
                assert_eq!(meta.id, id);
            }
        });
    });
}

criterion_group!(benches, bench_register_many, bench_lookup);
criterion_main!(benches);