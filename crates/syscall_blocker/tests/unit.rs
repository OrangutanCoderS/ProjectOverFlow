use syscall_blocker::{handle_request, SysBlockRequest};

#[test]
fn test_mock_block_syscall() {
    let req = SysBlockRequest {
        pid: 1000,
        syscall: "unlink".into(),
        mode: "enforce".into(),
    };
    assert!(handle_request(req).is_ok());
}