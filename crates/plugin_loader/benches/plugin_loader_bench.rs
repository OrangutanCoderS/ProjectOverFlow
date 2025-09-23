use criterion::{criterion_group, criterion_main, Criterion};
use plugin_loader::{LoaderConfig, PluginLoader, MockEngine};
use rand::{rngs::StdRng, Rng, SeedableRng};
use std::fs::File;
use std::io::Write;
use tempfile::TempDir;

fn make_lib(dir: &std::path::Path, name: &str, seed: u64) {
    let ext = if cfg!(target_os = "macos") { "dylib" }
    else if cfg!(target_os = "linux") { "so" }
    else if cfg!(target_os = "windows") { "dll" }
    else { "bin" };

    let mut p = dir.to_path_buf();
    p.push(format!("{name}.{ext}"));
    let mut f = File::create(&p).unwrap();
    let mut rng = StdRng::seed_from_u64(seed);
    let mut data = vec![0u8; 16 * 1024];
    rng.fill(&mut data[..]);
    f.write_all(&data).unwrap();
}

fn bench_discovery_filter(c: &mut Criterion) {
    let td = TempDir::new().unwrap();
    let root = td.path();

    // 200 fake libs + 100 text files
    for i in 0..200 {
        make_lib(root, &format!("p{i:03}"), i as u64);
    }
    for i in 0..100 {
        std::fs::write(root.join(format!("note{i:03}.txt")), b"-").unwrap();
    }

    let cfg = LoaderConfig {
        roots: vec![root.to_path_buf()],
        max_plugins: 150, // simulate policy limit
        ..Default::default()
    };
    let loader = PluginLoader::new(cfg, MockEngine);

    c.bench_function("discover+filter", |b| {
        b.iter(|| {
            let _ = loader.load_all().unwrap();
        })
    });
}

criterion_group!(benches, bench_discovery_filter);
criterion_main!(benches);
