use criterion::{criterion_group, criterion_main, Criterion, black_box};
use memory_patcher::{Signature, PatchSpecV1, apply_patch};
use std::time::Duration;

fn bench_scan_and_patch(c: &mut Criterion) {
    let sig = Signature::parse("48 8B 05 ?? ?? 55 8B").unwrap();
    c.bench_function("scan+dry_run_patch", |b| {
        b.iter(|| {
            let spec = PatchSpecV1 {
                pid: 1234,
                signature: sig.clone(),
                offset: 0,
                new_bytes: vec![0x90, 0x90, 0x90],
                region_hint: Some(".text".into()),
                checksum_before: None,
                checksum_after: None,
                dry_run: true,
                timeout: Duration::from_millis(10),
                rollback_on_fail: true,
            };
            let _ = apply_patch(black_box(&spec)).unwrap();
        })
    });
}

criterion_group!(benches, bench_scan_and_patch);
criterion_main!(benches);