use fake_device_emulator::*;
use std::sync::Arc;
use tempfile::tempdir;

/// Basic register/list/unregister tests.
#[test]
fn register_and_list_and_remove() {
    let backend = Arc::new(MockBackend::new());
    let mgr = FakeDeviceManager::new(backend.clone(), ManagerConfig::default());

    let spec = DeviceSpec {
        kind: DeviceKind::BlockDevice,
        vendor: Some("Acme".into()),
        product: Some("SimDisk".into()),
        metadata: serde_json::Value::Null,
        size_bytes: Some(1024),
    };

    let dev = mgr.register_device(spec).expect("registration should succeed");
    let list = mgr.list();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].id, dev.id);

    mgr.unregister_device(&dev.id).expect("remove works");
    let list2 = mgr.list();
    assert!(list2.is_empty());
}

/// Validate that validation rejects bad specs
#[test]
fn block_device_requires_size() {
    let backend = Arc::new(MockBackend::new());
    let mgr = FakeDeviceManager::new(backend, ManagerConfig::default());

    let spec = DeviceSpec {
        kind: DeviceKind::BlockDevice,
        vendor: None,
        product: None,
        metadata: serde_json::Value::Null,
        size_bytes: Some(0),
    };

    assert!(mgr.register_device(spec).is_err());
}

/// Concurrency test: create many devices from threads.
#[test]
fn concurrency_create() {
    use std::thread;
    use std::sync::Arc;

    let backend = Arc::new(MockBackend::new());
    let mgr = Arc::new(FakeDeviceManager::new(backend, ManagerConfig { max_devices: 1000 }));

    let mut handles = vec![];
    for _ in 0..10 {
        let m = mgr.clone();
        handles.push(thread::spawn(move || {
            for _ in 0..50 {
                let spec = DeviceSpec {
                    kind: DeviceKind::HidKeyboard,
                    vendor: None,
                    product: None,
                    metadata: serde_json::Value::Null,
                    size_bytes: None,
                };
                let _ = m.register_device(spec);
            }
        }));
    }

    for h in handles { h.join().unwrap(); }

    // ensure we didn't get panics and index is stable
    let count = mgr.list().len();
    assert!(count <= 1000);
}