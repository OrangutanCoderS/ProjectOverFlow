use std::path::PathBuf;
use tempfile::tempdir;

use overflow_core::{AnyEvent, FileAccessEvent, FileOp};
use filemon::{FileMonCfg, FileMonitor};

#[test]
fn fs_usage_parser_minimal_cases() {
    // We call the internal parser through a small shim (exposed by the lib in tests).
    // To keep the public API minimal, we re-import the helper via the crate path.
    fn parse(line: &str, roots: &[PathBuf]) -> Option<FileAccessEvent> {
        filemon::tests::parse_for_tests(line, roots)
    }

    let roots = vec![PathBuf::from("/etc")];
    let l1 = "12:34:56  open   /etc/hosts   curl.4242";
    let e1 = parse(l1, &roots).unwrap();
    assert_eq!(e1.path, "/etc/hosts");
    assert!(matches!(e1.op, FileOp::Open));

    let l2 = "12:34:57  unlink   /etc/hosts   Finder.123";
    let e2 = parse(l2, &roots).unwrap();
    assert!(matches!(e2.op, FileOp::Delete));
}

#[test]
fn polling_finds_create_modify_delete() {
    // Use only polling code path logic (helpers are public in crate for tests).
    use std::io::Write;

    let dir = tempdir().unwrap();
    let root = dir.path().to_path_buf();

    let mut baseline = filemon::tests::snapshot_tree_for_tests(&[root.clone()]).unwrap();

    // Create
    let f = root.join("x.txt");
    {
        let mut fh = std::fs::File::create(&f).unwrap();
        writeln!(fh, "a").unwrap();
    }
    let mut evts = Vec::new();
    let cur = filemon::tests::snapshot_tree_for_tests(&[root.clone()]).unwrap();
    filemon::tests::diff_for_tests(&baseline, &cur, &mut evts);
    baseline = cur;
    assert!(evts.iter().any(|e| matches!(e, AnyEvent::FileAccess(FileAccessEvent { op: FileOp::Open, .. }))));

    // Modify
    {
        let mut fh = std::fs::OpenOptions::new().append(true).open(&f).unwrap();
        writeln!(fh, "b").unwrap();
    }
    let mut evts2 = Vec::new();
    let cur = filemon::tests::snapshot_tree_for_tests(&[root.clone()]).unwrap();
    filemon::tests::diff_for_tests(&baseline, &cur, &mut evts2);
    baseline = cur;
    assert!(evts2.iter().any(|e| matches!(e, AnyEvent::FileAccess(FileAccessEvent { op: FileOp::Write, .. }))));

    // Delete
    std::fs::remove_file(&f).unwrap();
    let mut evts3 = Vec::new();
    let cur = filemon::tests::snapshot_tree_for_tests(&[root]).unwrap();
    filemon::tests::diff_for_tests(&baseline, &cur, &mut evts3);
    assert!(evts3.iter().any(|e| matches!(e, AnyEvent::FileAccess(FileAccessEvent { op: FileOp::Delete, .. }))));
}

#[test]
fn monitor_constructs_and_snapshots() {
    let cfg = FileMonCfg::default();
    let mon = FileMonitor::new(cfg);
    if mon.is_err() {
        // Non-macOS CI is fine.
        return;
    }
    let mon = mon.unwrap();
    let _ = mon.snapshot(); // Allow empty or some events; environment dependent.
}