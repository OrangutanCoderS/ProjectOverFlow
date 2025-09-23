//! Module 23 — Plugin Loader
//!
//! Responsibilities:
//! - Safe discovery of plugin files (platform-appropriate extensions).
//! - Policy filtering (allowlist/denylist, max count).
//! - Optional integrity allowlist (sha256 by file path).
//! - Delegates the *actual* dlopen + ABI checks to `plugin_engine`.
//!
//! Design:
//! - `Engine` trait decouples loader from engine; production uses `RealEngine`
//!   that calls `plugin_engine::load_plugin`. Tests/benches use `MockEngine`.
//! - No mutations of other crates; zero impact on existing files.

use anyhow::{Context, Result};
use chrono::Utc;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{HashMap, HashSet},
    ffi::OsStr,
    fs,
    io::Read,
    path::{Path, PathBuf},
};
use thiserror::Error;
use tracing::{debug, info, warn};
use walkdir::WalkDir;

/// Public surface returned to callers
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LoadedPlugin {
    pub name: String,
    pub version: String,
    pub abi: u32,
    pub path: PathBuf,
}

/// Loader configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoaderConfig {
    /// Directories to scan for plugins.
    pub roots: Vec<PathBuf>,
    /// Optional list of exact plugin basenames allowed (no extension).
    pub allowlist: Option<HashSet<String>>,
    /// Optional list of exact plugin basenames denied (no extension).
    pub denylist: Option<HashSet<String>>,
    /// Maximum number of plugins to load (after filters).
    pub max_plugins: usize,
    /// Optional map of absolute file paths -> expected sha256 hex (integrity allowlist).
    pub sha256_allowlist: Option<HashMap<PathBuf, String>>,
}

impl Default for LoaderConfig {
    fn default() -> Self {
        Self {
            roots: vec![PathBuf::from("plugins")],
            allowlist: None,
            denylist: None,
            max_plugins: 256,
            sha256_allowlist: None,
        }
    }
}

/// Errors specific to the loader phase (before engine load)
#[derive(Debug, Error)]
pub enum LoaderError {
    #[error("invalid root directory: {0}")]
    InvalidRoot(String),

    #[error("candidate rejected by policy: {0}")]
    Rejected(String),

    #[error("integrity check failed for {path}: expected {expected} got {got}")]
    Integrity {
        path: PathBuf,
        expected: String,
        got: String,
    },
}

/// Engine trait to abstract actual load logic.
pub trait Engine: Send + Sync + 'static {
    fn load(&self, path: PathBuf) -> Result<LoadedPlugin>;
}

/// Real engine adapter that calls `plugin_engine::load_plugin`.
pub struct RealEngine;

impl Engine for RealEngine {
    fn load(&self, path: PathBuf) -> Result<LoadedPlugin> {
        // Delegate to existing engine (no code changes there)
        let meta = plugin_engine::load_plugin(path.clone())
            .with_context(|| format!("engine failed to load {:?}", path))?;
        Ok(LoadedPlugin {
            name: meta.name,
            version: meta.version,
            abi: meta.abi,
            path,
        })
    }
}

/// The Loader
pub struct PluginLoader<E: Engine = RealEngine> {
    cfg: LoaderConfig,
    engine: E,
    loaded: Mutex<Vec<LoadedPlugin>>,
}

impl<E: Engine> PluginLoader<E> {
    pub fn new(cfg: LoaderConfig, engine: E) -> Self {
        Self {
            cfg,
            engine,
            loaded: Mutex::new(Vec::new()),
        }
    }

    /// Discover, filter, and load all plugins according to policy.
    pub fn load_all(&self) -> Result<Vec<LoadedPlugin>> {
        let candidates = self.discover_candidates()?;
        debug!("discovered {} candidates", candidates.len());

        let filtered = self.apply_policy(candidates)?;
        debug!("{} after policy filters", filtered.len());

        let mut out = Vec::new();
        for path in filtered.into_iter().take(self.cfg.max_plugins) {
            match self.engine.load(path.clone()) {
                Ok(loaded) => {
                    info!(plugin = %loaded.name, version = %loaded.version, abi = loaded.abi, "loaded");
                    out.push(loaded);
                }
                Err(err) => {
                    // We log and continue (one bad plugin shouldn't block others).
                    warn!(path = ?path, error = %err, "engine load failed");
                }
            }
        }

        // Persist the snapshot
        let mut guard = self.loaded.lock();
        *guard = out.clone();
        Ok(out)
    }

    /// Return the last successful loaded snapshot
    pub fn snapshot(&self) -> Vec<LoadedPlugin> {
        self.loaded.lock().clone()
    }

    /// Scan filesystem for platform-appropriate library files.
    fn discover_candidates(&self) -> Result<Vec<PathBuf>> {
        let mut out = Vec::new();
        for root in &self.cfg.roots {
            if !root.exists() {
                return Err(LoaderError::InvalidRoot(format!(
                    "root {:?} does not exist",
                    root
                ))
                .into());
            }
            if !root.is_dir() {
                return Err(LoaderError::InvalidRoot(format!(
                    "root {:?} is not a directory",
                    root
                ))
                .into());
            }

            for entry in WalkDir::new(root).follow_links(false).into_iter() {
                let entry = match entry {
                    Ok(e) => e,
                    Err(e) => {
                        warn!(error = %e, "walkdir error");
                        continue;
                    }
                };
                if !entry.file_type().is_file() {
                    continue;
                }
                let p = entry.path();
                if is_dynlib(p) {
                    out.push(p.to_path_buf());
                }
            }
        }
        Ok(out)
    }

    /// Apply allow/deny and integrity checks.
    fn apply_policy(&self, candidates: Vec<PathBuf>) -> Result<Vec<PathBuf>> {
        let mut out = Vec::new();

        for path in candidates {
            // basename without extension
            let Some(stem) = path.file_stem().and_then(OsStr::to_str) else {
                continue;
            };

            if let Some(deny) = &self.cfg.denylist {
                if deny.contains(stem) {
                    debug!(?path, "denied by denylist");
                    continue;
                }
            }
            if let Some(allow) = &self.cfg.allowlist {
                if !allow.contains(stem) {
                    debug!(?path, "rejected: not in allowlist");
                    continue;
                }
            }

            // Integrity allowlist if configured (only for exact paths present there)
            if let Some(map) = &self.cfg.sha256_allowlist {
                if let Some(expected) = map.get(&path) {
                    let got = sha256_file(&path)
                        .with_context(|| format!("hashing {:?}", path))?;
                    if &got != expected {
                        return Err(LoaderError::Integrity {
                            path: path.clone(),
                            expected: expected.clone(),
                            got,
                        }
                        .into());
                    }
                }
            }

            out.push(path);
        }

        Ok(out)
    }
}

/// OS-specific dynamic library extension check
fn is_dynlib(p: &Path) -> bool {
    let ext = p.extension().and_then(OsStr::to_str).unwrap_or_default().to_ascii_lowercase();
    #[cfg(target_os = "macos")]
    { ext == "dylib" }
    #[cfg(target_os = "linux")]
    { ext == "so" }
    #[cfg(target_os = "windows")]
    { ext == "dll" }
    #[cfg(all(not(target_os = "macos"), not(target_os = "linux"), not(target_os = "windows")))]
    { false }
}

/// Compute sha256 hex of a file.
fn sha256_file(p: &Path) -> Result<String> {
    let mut f = fs::File::open(p)?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 64 * 1024];
    loop {
        let n = f.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hex::encode(hasher.finalize()))
}

/* -----------------------
   Test-only Mock Engine
   ----------------------- */

#[cfg(any(test, feature = "test_mock_engine"))]
pub struct MockEngine;

#[cfg(any(test, feature = "test_mock_engine"))]
impl Engine for MockEngine {
    fn load(&self, path: PathBuf) -> Result<LoadedPlugin> {
        // fake metadata derived from filename for deterministic tests
        let name = path
            .file_stem()
            .and_then(OsStr::to_str)
            .unwrap_or("unknown")
            .to_string();
        Ok(LoadedPlugin {
            name,
            version: "0.0.1-test".to_string(),
            abi: 1,
            path,
        })
    }
}

/* -----------------------
   Mini self-check (opt-in)
   ----------------------- */
#[cfg(test)]
mod self_tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use rand::{rngs::StdRng, Rng, SeedableRng};
    use tempfile::TempDir;

    // helper to create a fake dynamic lib file with random bytes
    fn make_fake_lib(dir: &Path, name: &str, seed: u64) -> PathBuf {
        let ext = if cfg!(target_os = "macos") { "dylib" }
        else if cfg!(target_os = "linux") { "so" }
        else if cfg!(target_os = "windows") { "dll" }
        else { "bin" };

        let mut path = dir.to_path_buf();
        path.push(format!("{name}.{ext}"));
        let mut f = std::fs::File::create(&path).unwrap();
        let mut rng = StdRng::seed_from_u64(seed);
        let mut data = vec![0u8; 8192];
        rng.fill(&mut data[..]);
        std::io::Write::write_all(&mut f, &data).unwrap();
        path
    }

    #[test]
    fn discovery_and_policy() {
        let td = TempDir::new().unwrap();
        let root = td.path().to_path_buf();

        // good candidates
        let a = make_fake_lib(&root, "alpha", 1);
        let b = make_fake_lib(&root, "beta", 2);
        // non-dynlib file
        std::fs::write(root.join("readme.txt"), b"nope").unwrap();

        // allow only alpha
        let mut allow = HashSet::new();
        allow.insert("alpha".to_string());

        let cfg = LoaderConfig {
            roots: vec![root.clone()],
            allowlist: Some(allow),
            denylist: None,
            max_plugins: 10,
            sha256_allowlist: None,
        };
        let loader = PluginLoader::new(cfg, MockEngine);
        let loaded = loader.load_all().unwrap();

        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].name, "alpha");
        assert_eq!(loader.snapshot().len(), 1);

        // keep unused variables happy
        let _ = (a, b);
    }

    #[test]
    fn integrity_allowlist_enforced() {
        let td = TempDir::new().unwrap();
        let root = td.path().to_path_buf();

        let p = make_fake_lib(&root, "gamma", 7);
        let good_hash = sha256_file(&p).unwrap();

        // wrong hash on purpose
        let mut map = HashMap::new();
        map.insert(p.clone(), "deadbeef".repeat(8)); // 64 hex chars

        let cfg = LoaderConfig {
            roots: vec![root.clone()],
            allowlist: None,
            denylist: None,
            max_plugins: 5,
            sha256_allowlist: Some(map),
        };
        let loader = PluginLoader::new(cfg, MockEngine);

        // should fail integrity before engine load
        let err = loader.load_all().unwrap_err().to_string();
        assert!(err.contains("integrity check failed"));

        // sanity: correct hash passes
        let mut ok = HashMap::new();
        ok.insert(p.clone(), good_hash);
        let cfg_ok = LoaderConfig { sha256_allowlist: Some(ok), ..LoaderConfig { roots: vec![root], ..Default::default() } };
        let loader_ok = PluginLoader::new(cfg_ok, MockEngine);
        let res = loader_ok.load_all().unwrap();
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].name, "gamma");
    }

    #[test]
    fn max_plugins_respected() {
        let td = TempDir::new().unwrap();
        let root = td.path().to_path_buf();

        let _ = make_fake_lib(&root, "p1", 1);
        let _ = make_fake_lib(&root, "p2", 2);
        let _ = make_fake_lib(&root, "p3", 3);

        let cfg = LoaderConfig {
            roots: vec![root],
            max_plugins: 2,
            ..Default::default()
        };
        let loader = PluginLoader::new(cfg, MockEngine);
        let res = loader.load_all().unwrap();
        assert_eq!(res.len(), 2);
    }
}
