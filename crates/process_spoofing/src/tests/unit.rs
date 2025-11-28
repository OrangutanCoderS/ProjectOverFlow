use process_spoofing::*;
use std::{collections::HashMap, thread, time::Duration};

#[test]
fn add_and_query_spoof() {
    let mgr = SpoofManager::new();
    let mut env = HashMap::new();
    env.insert("USER".into(), "root".into());
    let s = SpoofIdentity {
        real_pid: 111,
        fake_pid: 999,
        fake_ppid: 1,
        fake_name: Some("fakeproc".into()),
        fake_env: Some(env.clone()),
        created_at: chrono::Utc::now(),
        ttl: Some(Duration::from_secs(5)),
    };
    mgr.add_spoof(s.clone()).unwrap();
    assert_eq!(mgr.query(111).unwrap().fake_pid, 999);
    thread::sleep(Duration::from_secs(1));
    mgr.cleanup_expired();
    assert!(mgr.query(111).is_some());
}