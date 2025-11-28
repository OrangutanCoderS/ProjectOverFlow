use criterion::{criterion_group, criterion_main, Criterion};
use std::sync::Arc;
use fake_device_emulator::*;
use rayon::prelude::*; // for easy parallel iteration

fn bench_register_block(c: &mut Criterion) {
    let mgr = FakeDeviceManager::new(Arc::new(MockBackend::new()), ManagerConfig::default());

    c.bench_function("register_block_device", |b| {
        b.iter(|| {
            let spec = DeviceSpec {
                kind: DeviceKind::BlockDevice,
                vendor: Some("bench_vendor".into()),
                product: Some("bench_block".into()),
                metadata: serde_json::Value::Null,
                size_bytes: Some(1024 * 1024),
            };
            let _ = mgr.register_device(spec);
        })
    });
}

fn bench_register_keyboard(c: &mut Criterion) {
    let mgr = FakeDeviceManager::new(Arc::new(MockBackend::new()), ManagerConfig::default());

    c.bench_function("register_hid_keyboard", |b| {
        b.iter(|| {
            let spec = DeviceSpec {
                kind: DeviceKind::HidKeyboard,
                vendor: Some("bench_vendor".into()),
                product: Some("bench_keyboard".into()),
                metadata: serde_json::Value::Null,
                size_bytes: None,
            };
            let _ = mgr.register_device(spec);
        })
    });
}

/// 🔥 Concurrency stress test
fn bench_concurrent_register(c: &mut Criterion) {
    let mgr = Arc::new(FakeDeviceManager::new(
        Arc::new(MockBackend::new()),
        ManagerConfig { max_devices: 10_000 },
    ));

    c.bench_function("concurrent_register_block", |b| {
        b.iter(|| {
            // Simulate 100 parallel device creations
            (0..100).into_par_iter().for_each(|i| {
                let spec = DeviceSpec {
                    kind: DeviceKind::BlockDevice,
                    vendor: Some("bench_vendor".into()),
                    product: Some(format!("bench_block_{}", i)),
                    metadata: serde_json::Value::Null,
                    size_bytes: Some(1024 * 1024),
                };
                let _ = mgr.register_device(spec);
            });
            mgr.cleanup(); // reset state after each iteration
        })
    });
}

criterion_group!(
    benches,
    bench_register_block,
    bench_register_keyboard,
    bench_concurrent_register
);
criterion_main!(benches);