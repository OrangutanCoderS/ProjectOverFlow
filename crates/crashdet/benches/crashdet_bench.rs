use criterion::{criterion_group, criterion_main, Criterion, black_box};
use std::fs;
use std::io::Write;
use tempfile::tempdir;
use crashdet::{CrashConfig, poll_crashes};

fn bench_poll_parse_small(c: &mut Criterion) {
    // Create a tiny directory with a few crash files
    let td = tempdir().unwrap();
    for i in 0..10 {
        let p = td.path().join(format!("App{}.crash", i));
        let mut f = fs::File::create(&p).unwrap();
        writeln!(f, "Process:\tApp{} [{}]", i, 1000 + i).unwrap();
        writeln!(f, "Exception Type:\tEXC_BAD_ACCESS (SIGSEGV)").unwrap();
    }

    let mut cfg = CrashConfig::default();
    cfg.crash_dirs = vec![td.path().to_path_buf()];
    cfg.max_entries_per_sample = 10;

    c.bench_function("crashdet_poll_small", |b| {
        b.iter(|| {
            let out = poll_crashes(black_box(&cfg)).unwrap();
            black_box(out);
        })
    });
}

criterion_group!(benches, bench_poll_parse_small);
criterion_main!(benches);
