use syscall_interceptor::{SyscallEvent, SyscallInterceptor};
use chrono::Utc;

#[test]
fn attach_and_detach() {
    let mut icp = SyscallInterceptor::new();
    assert!(icp.attach().is_ok());
    assert!(icp.detach().is_ok());
}

#[test]
fn push_and_recv_mock() {
    let mut icp = SyscallInterceptor::new();
    icp.attach().unwrap();

    let ev = SyscallEvent {
        pid: 123,
        name: "open".into(),
        args: vec!["/tmp/test.txt".into()],
        timestamp: Utc::now().to_rfc3339(),
    };

    icp.push_mock(ev.clone()).unwrap();
    let recv = icp.try_recv().unwrap();
    assert_eq!(recv, Some(ev));
}