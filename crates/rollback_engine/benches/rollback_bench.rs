use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rollback_engine::RollbackEngine;
use std::fs;
use std::path::PathBuf;
use chrono::Utc;
use serde_json::json;

/// Utility: create a fake snapshot directory structure.
fn make_fake_snapshot(root: &PathBuf, id: &str, file_count: usize) -> PathBuf {
    let snapshot_dir = root.join(id);
    fs::create_dir_all(&snapshot_dir).unwrap();

    let mut files = Vec::new();

    for n in 0..file_count {
        let rel = format!("file_{}.txt", n);
        let path = snapshot_dir.join(&rel);
        fs::write(&path, format!("dummy-data-{}", n)).unwrap();

        let sha = fake_sha256(&format!("dummy-data-{}", n));
        files.push(json!({
            "relative_path": rel,
            "sha256": sha,
        }));
    }

    // metadata
    let meta = json!({
        "id": id,
        "timestamp": Utc::now(),
        "branching_policy": "branch.yaml",
        "plugin_graph": "graph.json",
        "memory_snapshot": "mem.json",
        "execution_limiter": null,
        "files": files
    });

    fs::write(snapshot_dir.join("snapshot_meta.json"), meta.to_string()).unwrap();

    snapshot_dir
}

/// Cheap hash substitute (not cryptographically correct, but deterministic for bench)
fn fake_sha256(s: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(s.as_bytes());
    let out = h.finalize();
    out.iter().map(|b| format!("{:02x}", b)).collect()
}

pub fn bench_rollback(c: &mut Criterion) {
    let base = PathBuf::from("/tmp/rollback_bench_root");
    let _ = fs::remove_dir_all(&base);
    fs::create_dir_all(&base).unwrap();

    let snapshot_root = base.join("snapshots");
    let policy_root = base.join("policies");

    fs::create_dir_all(&snapshot_root).unwrap();
    fs::create_dir_all(&policy_root).unwrap();

    let engine = RollbackEngine::new(snapshot_root.clone(), policy_root.clone(), None);

    // Build a snapshot with 20 small files
    make_fake_snapshot(&snapshot_root, "snap-20", 20);

    // ---- BENCH validate_integrity ----
    c.bench_function("rollback_validate_integrity_20_files", |b| {
        b.iter(|| {
            let _ = engine.validate_integrity(black_box("snap-20"));
        });
    });

    // ---- BENCH simulate_revert ----
    c.bench_function("rollback_simulate_revert_20_files", |b| {
        b.iter(|| {
            let _ = engine.simulate_revert(black_box("snap-20"));
        });
    });

    // ---- BENCH rollback_to (disk-heavy, but controlled) ----
    c.bench_function("rollback_apply_20_files", |b| {
        b.iter(|| {
            let _ = engine.rollback_to(
                black_box("snap-20"),
                black_box("bench-operator"),
                None,
            );
        });
    });
}

criterion_group!(benches, bench_rollback);
criterion_main!(benches);