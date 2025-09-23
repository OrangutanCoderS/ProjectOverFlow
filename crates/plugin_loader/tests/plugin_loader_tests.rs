use plugin_loader::{LoaderConfig, PluginLoader, MockEngine};
use pretty_assertions::assert_eq;
use rand::{rngs::StdRng, Rng, SeedableRng};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use tempfile::TempDir;

fn dyn_ext() -> &'static str {
    if cfg!(target_os = "macos") { "dylib" }
    else if cfg!(target_os = "linux") { "so" }
    else if cfg!(target_os = "windows") { "dll" }
    else { "bin" }
}

fn make_lib(dir: &std::path::Path, name: &str, seed: u64) -> PathBuf {
    let mut p = dir.to_path_buf();
    p.push(format!("{name}.{}", dyn_ext()));
    let mut f = File::create(&p).unwrap();
    let mut rng = StdRng::seed_from_u64(seed);
    let mut data = vec![0u8; 4096];
    rng.fill(&mut data[..]);
    f.write_all(&data).unwrap();
    p
}

#[test]
fn allow_and_deny() {
    let td = TempDir::new().unwrap();
    let root = td.path();

    let _ = make_lib(root, "keep_me", 1);
    let _ = make_lib(root, "drop_me", 2);

    let mut allow = HashSet::new();
    allow.insert("keep_me".to_string());
    let mut deny = HashSet::new();
    deny.insert("drop_me".to_string());

    let cfg = LoaderConfig {
        roots: vec![root.to_path_buf()],
        allowlist: Some(allow),
        denylist: Some(deny),
        max_plugins: 10,
        sha256_allowlist: None,
    };

    let loader = PluginLoader::new(cfg, MockEngine);
    let out = loader.load_all().unwrap();
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].name, "keep_me");
}

#[test]
fn integrity_map_exact_path() {
    let td = TempDir::new().unwrap();
    let root = td.path();

    let lib = make_lib(root, "sigged", 42);
    let hash = plugin_loader::sha256_file(&lib).unwrap();

    // wrong path -> not enforced
    let mut m = HashMap::new();
    m.insert(root.join("other").join("sigged.so"), hash.clone());

    let cfg = LoaderConfig {
        roots: vec![root.to_path_buf()],
        sha256_allowlist: Some(m),
        ..Default::default()
    };

    let loader = PluginLoader::new(cfg, MockEngine);
    let out = loader.load_all().unwrap();
    assert_eq!(out.len(), 1);
}
