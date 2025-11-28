use std::path::PathBuf;

use rollback_engine::RollbackEngine;

#[test]
fn engine_constructs() {
    let snapshots = PathBuf::from("/tmp/overflow_snapshots");
    let policies = PathBuf::from("/tmp/overflow_policies");
    let audit = None;

    let engine = RollbackEngine::new(snapshots, policies, audit);
    // No panic, no type errors – that's all this test guarantees for now.
    assert!(engine.snapshot_root().to_str().is_some());
}