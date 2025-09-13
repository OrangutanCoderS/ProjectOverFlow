use criterion::{criterion_group, criterion_main, Criterion};

fn bench_parser(c: &mut Criterion) {
    let roots = vec![std::path::PathBuf::from("/etc")];
    let line = "12:00:00  open  /etc/hosts   curl.4242";

    c.bench_function("parse_fs_usage_line", |b| {
        b.iter(|| {
            let _ = filemon::test_shims::parse_for_tests(line, &roots);
        })
    });
}

fn bench_diff(c: &mut Criterion) {
    use std::io::Write;
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().to_path_buf();

    // build small trees
    let baseline = filemon::test_shims::snapshot_tree_for_tests(&[root.clone()]).unwrap();
    let f = root.join("a.txt");
    { let mut fh = std::fs::File::create(&f).unwrap(); writeln!(fh, "x").unwrap(); }
    let current = filemon::test_shims::snapshot_tree_for_tests(&[root.clone()]).unwrap();

    c.bench_function("diff_snapshots_small", |b| {
        b.iter(|| {
            let mut out = Vec::new();
            filemon::test_shims::diff_for_tests(&baseline, &current, &mut out);
        })
    });
}

criterion_group!(benches, bench_parser, bench_diff);
criterion_main!(benches);