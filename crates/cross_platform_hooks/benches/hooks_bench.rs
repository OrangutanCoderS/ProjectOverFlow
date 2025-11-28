use std::fs;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use cross_platform_hooks::linux::LinuxPortConfigLoaderFs;
use cross_platform_hooks::{DefaultPolicy, LinuxPortConfig, LinuxPortRule, Protocol};
use tempfile::tempdir;

fn bench_linux_yaml_parse(c: &mut Criterion) {
    let dir = tempdir().unwrap();
    let path = dir.path().join("linux_port_config.yaml");

    // Build a large config in memory and serialize to YAML once.
    let mut rules = Vec::with_capacity(2000);
    for p in 1u16..=2000 {
        rules.push(LinuxPortRule {
            port: p,
            protocol: Protocol::Tcp,
            tag: format!("svc-{}", p),
            allow: p % 2 == 0,
            note: None,
        });
    }

    let cfg = LinuxPortConfig {
        rules,
        default_policy: DefaultPolicy::Deny,
    };

    let yaml = serde_yaml::to_string(&cfg).unwrap();
    fs::write(&path, yaml).unwrap();

    c.bench_function("linux_port_config_parse_2000_rules", |b| {
        b.iter(|| {
            let loader = LinuxPortConfigLoaderFs::new(&path);
            let cfg = loader.load().unwrap();
            black_box(cfg.rules.len());
        });
    });
}

criterion_group!(benches, bench_linux_yaml_parse);
criterion_main!(benches);
